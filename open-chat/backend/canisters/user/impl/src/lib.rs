use crate::model::communities::Communities;
use crate::model::community::Community;
use crate::model::direct_chats::DirectChats;
use crate::model::group_chat::GroupChat;
use crate::model::group_chats::GroupChats;
use crate::model::hot_group_exclusions::HotGroupExclusions;
use crate::model::local_user_index_event_batch::LocalUserIndexEventBatch;
use crate::model::p2p_swaps::P2PSwaps;
use crate::model::pin_number::PinNumber;
use crate::model::token_swaps::TokenSwaps;
use crate::model::user_canister_event_batch::UserCanisterEventBatch;
use crate::timer_job_types::{ClaimOrResetStreakInsuranceJob, DeleteFileReferencesJob, RemoveExpiredEventsJob, TimerJob};
use candid::Principal;
use canister_state_macros::canister_state;
use canister_timer_jobs::{Job, TimerJobs};
use chat_events::EventPusher;
use constants::{DAY_IN_MS, ICP_LEDGER_CANISTER_ID, LIFETIME_DIAMOND_TIMESTAMP, OPENCHAT_BOT_USER_ID};
use event_store_types::{Event, EventBuilder};
use fire_and_forget_handler::FireAndForgetHandler;
use installed_bots::InstalledBots;
use local_user_index_canister::UserEvent as LocalUserIndexEvent;
use model::chit_earned_events::ChitEarnedEvents;
use model::contacts::Contacts;
use model::favourite_chats::FavouriteChats;
use model::message_activity_events::MessageActivityEvents;
use model::referrals::Referrals;
use model::streak::Streak;
use msgpack::serialize_then_unwrap;
use oc_error_codes::OCErrorCode;
use rand::RngCore;
use rand::prelude::StdRng;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use stable_memory_map::{BaseKeyPrefix, ChatEventKeyPrefix};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::ops::Deref;
use timer_job_queues::{BatchedTimerJobQueue, GroupedTimerJobQueue};
use types::{
    Achievement, BotInitiator, BotNotification, BotPermissions, BuildVersion, CanisterId, Chat, ChatId, ChatMetrics,
    ChitEarned, ChitEarnedReason, CommunityId, Cycles, Document, FcmData, IdempotentEnvelope, Milliseconds, Notification,
    NotifyChit, TimestampMillis, Timestamped, UniquePersonProof, UserCanisterStreakInsuranceClaim,
    UserCanisterStreakInsurancePayment, UserId, UserNotification, UserNotificationPayload,
};
use user_canister::{MessageActivityEvent, NamedAccount, UserCanisterEvent, WalletConfig};
use utils::env::Environment;
use utils::idempotency_checker::IdempotencyChecker;
use utils::regular_jobs::RegularJobs;

mod crypto;
mod governance_clients;
mod guards;
mod jobs;
mod lifecycle;
mod memory;
mod model;
mod openchat_bot;
mod queries;
mod regular_jobs;
mod timer_job_types;
mod token_swaps;
mod updates;

pub const COMMUNITY_CREATION_LIMIT: u32 = 10;
const SIX_MONTHS: Milliseconds = 183 * DAY_IN_MS;

thread_local! {
    static WASM_VERSION: RefCell<Timestamped<BuildVersion>> = RefCell::default();
}

canister_state!(RuntimeState);

struct RuntimeState {
    pub env: Box<dyn Environment>,
    pub data: Data,
    pub regular_jobs: RegularJobs<Data>,
}

impl RuntimeState {
    pub fn new(env: Box<dyn Environment>, data: Data, regular_jobs: RegularJobs<Data>) -> RuntimeState {
        RuntimeState { env, data, regular_jobs }
    }

    pub fn is_caller_owner(&self) -> bool {
        self.env.caller() == self.data.owner
    }

    pub fn is_caller_user_index(&self) -> bool {
        self.env.caller() == self.data.user_index_canister_id
    }

    pub fn is_caller_local_user_index(&self) -> bool {
        self.env.caller() == self.data.local_user_index_canister_id
    }

    pub fn is_caller_group_index(&self) -> bool {
        self.env.caller() == self.data.group_index_canister_id
    }

    pub fn is_caller_escrow_canister(&self) -> bool {
        self.env.caller() == self.data.escrow_canister_id
    }

    pub fn is_caller_known_group_canister(&self) -> bool {
        let caller = self.env.caller();
        self.data.group_chats.exists(&caller.into())
    }

    pub fn is_caller_known_community_canister(&self) -> bool {
        let caller = self.env.caller();
        self.data.communities.exists(&caller.into())
    }

    pub fn is_caller_video_call_operator(&self) -> bool {
        let caller = self.env.caller();
        self.data.video_call_operators.contains(&caller)
    }

    pub fn push_notification(
        &mut self,
        sender: Option<UserId>,
        recipient: UserId,
        notification: UserNotificationPayload,
        fcm_data: FcmData,
    ) {
        self.data.local_user_index_event_sync_queue.push(IdempotentEnvelope {
            created_at: self.env.now(),
            idempotency_id: self.env.rng().next_u64(),
            value: local_user_index_canister::UserEvent::Notification(Box::new(Notification::User(UserNotification {
                sender,
                recipients: vec![recipient],
                notification_bytes: ByteBuf::from(serialize_then_unwrap(notification)),
                fcm_data: Some(fcm_data),
            }))),
        })
    }

    pub fn run_event_expiry_job(&mut self) {
        let now = self.env.now();
        let mut next_event_expiry = None;
        let mut files_to_delete = Vec::new();
        for chat in self.data.direct_chats.iter_mut() {
            let result = chat.events.remove_expired_events(now);
            if let Some(expiry) = chat.events.next_event_expiry() {
                if next_event_expiry.is_none_or(|current| expiry < current) {
                    next_event_expiry = Some(expiry);
                }
            }
            files_to_delete.extend(result.files);
        }

        if !files_to_delete.is_empty() {
            let delete_files_job = DeleteFileReferencesJob { files: files_to_delete };
            delete_files_job.execute();
        }
        self.data.next_event_expiry = next_event_expiry;
        if let Some(expiry) = self.data.next_event_expiry {
            self.data
                .timer_jobs
                .enqueue_job(TimerJob::RemoveExpiredEvents(RemoveExpiredEventsJob), expiry, now);
        }
    }

    pub fn push_user_canister_event(&mut self, canister_id: CanisterId, event: UserCanisterEvent) {
        if canister_id != OPENCHAT_BOT_USER_ID.into() && canister_id != self.env.canister_id() {
            self.data.user_canister_events_queue.push(
                canister_id.into(),
                IdempotentEnvelope {
                    created_at: self.env.now(),
                    idempotency_id: self.env.rng().next_u64(),
                    value: event,
                },
            );
        }
    }

    pub fn mark_streak_insurance_payment(&mut self, payment: UserCanisterStreakInsurancePayment) {
        self.data.streak.mark_streak_insurance_payment(payment.clone());
        self.set_up_streak_insurance_timer_job();
        let user_id: UserId = self.env.canister_id().into();
        let events = vec![
            LocalUserIndexEvent::EventStoreEvent(
                EventBuilder::new("user_streak_insurance_payment", payment.timestamp)
                    .with_user(user_id.to_string(), true)
                    .with_source(user_id.to_string(), true)
                    .with_json_payload(&payment)
                    .build(),
            ),
            LocalUserIndexEvent::NotifyStreakInsurancePayment(payment),
        ];
        self.push_local_user_index_canister_events(events, self.env.now());
    }

    pub fn mark_streak_insurance_claim(&mut self, claim: UserCanisterStreakInsuranceClaim) {
        self.data.chit_events.push(ChitEarned {
            amount: 0,
            timestamp: claim.timestamp,
            reason: ChitEarnedReason::StreakInsuranceClaim,
        });
        let user_id: UserId = self.env.canister_id().into();
        let new_streak = claim.streak_length;
        let days_remaining = claim.insured_days_remaining;
        let events = vec![
            LocalUserIndexEvent::EventStoreEvent(
                EventBuilder::new("user_streak_insurance_claim", claim.timestamp)
                    .with_user(user_id.to_string(), true)
                    .with_source(user_id.to_string(), true)
                    .with_json_payload(&claim)
                    .build(),
            ),
            LocalUserIndexEvent::NotifyStreakInsuranceClaim(claim),
        ];
        self.push_local_user_index_canister_events(events, self.env.now());
        let days_remaining_text = if days_remaining == 1 { "1 day".to_string() } else { format!("{days_remaining} days") };
        openchat_bot::send_text_message(
            format!(
                "One day of streak insurance was just used up to protect your streak from being lost.\
Your streak is now {new_streak} days and you have {days_remaining_text} of streak insurance remaining."
            ),
            Vec::new(),
            false,
            self,
        );
    }

    pub fn set_up_streak_insurance_timer_job(&mut self) {
        if self.data.streak.days_insured() > 0 {
            self.data
                .timer_jobs
                .cancel_jobs(|j| matches!(j, TimerJob::ClaimOrResetStreakInsurance(_)));

            self.data.timer_jobs.enqueue_job(
                TimerJob::ClaimOrResetStreakInsurance(ClaimOrResetStreakInsuranceJob),
                self.data.streak.ends(),
                self.env.now(),
            );
        }
    }

    pub fn is_empty_and_dormant(&self) -> bool {
        if self.data.direct_chats.len() <= 1
            && self.data.group_chats.len() == 0
            && self.data.communities.len() == 0
            && self.data.diamond_membership_expires_at.is_none()
            && self.data.unique_person_proof.is_none()
            && self.data.group_chats.removed_len() == 0
            && self.data.communities.removed_len() == 0
        {
            let now = self.env.now();
            if self.data.user_created + SIX_MONTHS < now && self.data.chit_events.last_updated() + SIX_MONTHS < now {
                return true;
            }
        }
        false
    }

    pub fn push_bot_notification(&mut self, notification: Option<BotNotification>) {
        if let Some(notification) = notification {
            if !notification.recipients.is_empty() {
                self.push_local_user_index_canister_event(
                    LocalUserIndexEvent::Notification(Box::new(Notification::Bot(notification))),
                    self.env.now(),
                );
            }
        }
    }

    pub fn push_local_user_index_canister_event(&mut self, event: LocalUserIndexEvent, now: TimestampMillis) {
        self.data.local_user_index_event_sync_queue.push(IdempotentEnvelope {
            created_at: now,
            idempotency_id: self.env.rng().next_u64(),
            value: event,
        });
    }

    pub fn push_local_user_index_canister_events(&mut self, events: Vec<LocalUserIndexEvent>, now: TimestampMillis) {
        self.data.local_user_index_event_sync_queue.push_many(
            events
                .into_iter()
                .map(|event| IdempotentEnvelope {
                    created_at: now,
                    idempotency_id: self.env.rng().next_u64(),
                    value: event,
                })
                .collect(),
        );
    }

    pub fn award_achievements_and_notify(&mut self, achievements: Vec<Achievement>, now: TimestampMillis) {
        let mut awarded = false;

        for achievement in achievements {
            awarded |= self.data.award_achievement(achievement, now);
        }

        if awarded {
            self.notify_user_index_of_chit(now);
        }
    }

    pub fn award_achievement_and_notify(&mut self, achievement: Achievement, now: TimestampMillis) {
        if self.data.award_achievement(achievement, now) {
            self.notify_user_index_of_chit(now);
        }
    }

    pub fn award_external_achievement(&mut self, name: String, chit_reward: u32, now: TimestampMillis) -> bool {
        if self.data.external_achievements.insert(name.clone()) {
            self.data.chit_events.push(ChitEarned {
                amount: chit_reward as i32,
                timestamp: now,
                reason: ChitEarnedReason::ExternalAchievement(name),
            });

            self.notify_user_index_of_chit(now);

            true
        } else {
            false
        }
    }

    pub fn notify_user_index_of_chit(&mut self, now: TimestampMillis) {
        self.push_local_user_index_canister_event(
            LocalUserIndexEvent::NotifyChit(NotifyChit {
                timestamp: now,
                chit_balance: self.data.chit_events.balance_for_month_by_timestamp(now),
                streak: self.data.streak.days(now),
                streak_ends: self.data.streak.ends(),
            }),
            now,
        )
    }

    pub fn block_user(&mut self, user_id: UserId, now: TimestampMillis) {
        if self.data.blocked_users.value.insert(user_id) {
            self.data.blocked_users.timestamp = now;
            self.push_local_user_index_canister_event(LocalUserIndexEvent::UserBlocked(user_id), now);
        }
    }

    pub fn unblock_user(&mut self, user_id: UserId, now: TimestampMillis) {
        if self.data.blocked_users.value.remove(&user_id) {
            self.data.blocked_users.timestamp = now;
            self.push_local_user_index_canister_event(LocalUserIndexEvent::UserUnblocked(user_id), now);
        }
    }

    pub fn metrics(&self) -> Metrics {
        let now = self.env.now();
        Metrics {
            heap_memory_used: utils::memory::heap(),
            stable_memory_used: utils::memory::stable(),
            now,
            cycles_balance: self.env.cycles_balance(),
            liquid_cycles_balance: self.env.liquid_cycles_balance(),
            wasm_version: WASM_VERSION.with_borrow(|v| **v),
            git_commit_id: utils::git::git_commit_id().to_string(),
            direct_chats: self.data.direct_chats.len() as u32,
            group_chats: self.data.group_chats.len() as u32,
            communities: self.data.communities.len() as u32,
            groups_created: self.data.group_chats.groups_created(),
            blocked_users: self.data.blocked_users.len() as u32,
            created: self.data.user_created,
            direct_chat_metrics: self.data.direct_chats.metrics().hydrate(),
            video_call_operators: self.data.video_call_operators.clone(),
            timer_jobs: self.data.timer_jobs.len() as u32,
            queued_user_events: self.data.user_canister_events_queue.len() as u32,
            queued_local_index_events: self.data.local_user_index_event_sync_queue.len() as u32,
            chit_balance: self.data.chit_events.balance_for_month_by_timestamp(now),
            streak: self.data.streak.days(now),
            streak_ends: self.data.streak.ends(),
            max_streak: self.data.streak.max_streak(),
            next_daily_claim: self.data.streak.next_claim(),
            achievements: self.data.achievements.iter().cloned().collect(),
            unique_person_proof: self.data.unique_person_proof.is_some(),
            referred_by: self.data.referred_by,
            stable_memory_sizes: memory::memory_sizes(),
            canister_ids: CanisterIds {
                user_index: self.data.user_index_canister_id,
                group_index: self.data.group_index_canister_id,
                local_user_index: self.data.local_user_index_canister_id,
                identity: self.data.identity_canister_id,
                escrow: self.data.escrow_canister_id,
                icp_ledger: ICP_LEDGER_CANISTER_ID,
            },
        }
    }

    pub fn delete_direct_chat(&mut self, user_id: UserId, block_user: bool, now: TimestampMillis) -> bool {
        let Some(chat) = self.data.direct_chats.remove(user_id.into(), now) else {
            return false;
        };

        if block_user {
            self.block_user(user_id, now);
        }

        self.data
            .stable_memory_keys_to_garbage_collect
            .push(BaseKeyPrefix::from(ChatEventKeyPrefix::new_from_direct_chat(user_id, None)));

        for message_index in chat.events.thread_keys() {
            self.data.stable_memory_keys_to_garbage_collect.push(BaseKeyPrefix::from(
                ChatEventKeyPrefix::new_from_direct_chat(user_id, Some(message_index)),
            ));
        }

        jobs::garbage_collect_stable_memory::start_job_if_required(self);
        true
    }
}

#[derive(Serialize, Deserialize)]
struct Data {
    pub owner: Principal,
    pub direct_chats: DirectChats,
    pub group_chats: GroupChats,
    pub communities: Communities,
    pub favourite_chats: FavouriteChats,
    pub blocked_users: Timestamped<HashSet<UserId>>,
    pub user_index_canister_id: CanisterId,
    pub local_user_index_canister_id: CanisterId,
    pub group_index_canister_id: CanisterId,
    #[serde(default = "CanisterId::anonymous")]
    pub identity_canister_id: CanisterId,
    pub escrow_canister_id: CanisterId,
    pub avatar: Timestamped<Option<Document>>,
    pub test_mode: bool,
    pub is_platform_moderator: bool,
    pub hot_group_exclusions: HotGroupExclusions,
    pub username: Timestamped<String>,
    pub display_name: Timestamped<Option<String>>,
    pub bio: Timestamped<String>,
    pub storage_limit: u64,
    pub phone_is_verified: bool,
    pub user_created: TimestampMillis,
    pub suspended: Timestamped<bool>,
    pub timer_jobs: TimerJobs<TimerJob>,
    pub contacts: Contacts,
    pub diamond_membership_expires_at: Option<TimestampMillis>,
    pub fire_and_forget_handler: FireAndForgetHandler,
    pub saved_crypto_accounts: Vec<NamedAccount>,
    pub next_event_expiry: Option<TimestampMillis>,
    pub token_swaps: TokenSwaps,
    pub p2p_swaps: P2PSwaps,
    pub user_canister_events_queue: GroupedTimerJobQueue<UserCanisterEventBatch>,
    pub video_call_operators: Vec<Principal>,
    pub pin_number: PinNumber,
    pub btc_address: Option<Timestamped<String>>,
    pub chit_events: ChitEarnedEvents,
    pub streak: Streak,
    pub achievements: HashSet<Achievement>,
    pub external_achievements: HashSet<String>,
    pub achievements_last_seen: TimestampMillis,
    pub unique_person_proof: Option<UniquePersonProof>,
    pub wallet_config: Timestamped<WalletConfig>,
    pub rng_seed: [u8; 32],
    pub referred_by: Option<UserId>,
    pub referrals: Referrals,
    pub message_activity_events: MessageActivityEvents,
    pub stable_memory_keys_to_garbage_collect: Vec<BaseKeyPrefix>,
    pub local_user_index_event_sync_queue: BatchedTimerJobQueue<LocalUserIndexEventBatch>,
    pub idempotency_checker: IdempotencyChecker,
    pub bots: InstalledBots,
}

impl Data {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        owner: Principal,
        user_index_canister_id: CanisterId,
        local_user_index_canister_id: CanisterId,
        group_index_canister_id: CanisterId,
        identity_canister_id: CanisterId,
        escrow_canister_id: CanisterId,
        video_call_operators: Vec<Principal>,
        username: String,
        test_mode: bool,
        referred_by: Option<UserId>,
        now: TimestampMillis,
    ) -> Data {
        Data {
            owner,
            direct_chats: DirectChats::default(),
            group_chats: GroupChats::default(),
            communities: Communities::default(),
            favourite_chats: FavouriteChats::default(),
            blocked_users: Timestamped::default(),
            user_index_canister_id,
            local_user_index_canister_id,
            group_index_canister_id,
            identity_canister_id,
            escrow_canister_id,
            avatar: Timestamped::default(),
            test_mode,
            is_platform_moderator: false,
            hot_group_exclusions: HotGroupExclusions::default(),
            username: Timestamped::new(username, now),
            display_name: Timestamped::default(),
            bio: Timestamped::new("".to_string(), now),
            storage_limit: 0,
            phone_is_verified: false,
            user_created: now,
            suspended: Timestamped::default(),
            timer_jobs: TimerJobs::default(),
            contacts: Contacts::default(),
            diamond_membership_expires_at: None,
            fire_and_forget_handler: FireAndForgetHandler::default(),
            saved_crypto_accounts: Vec::new(),
            next_event_expiry: None,
            token_swaps: TokenSwaps::default(),
            p2p_swaps: P2PSwaps::default(),
            user_canister_events_queue: GroupedTimerJobQueue::new(10, true),
            video_call_operators,
            pin_number: PinNumber::default(),
            btc_address: None,
            chit_events: ChitEarnedEvents::default(),
            streak: Streak::default(),
            achievements: HashSet::new(),
            external_achievements: HashSet::new(),
            achievements_last_seen: 0,
            unique_person_proof: None,
            rng_seed: [0; 32],
            wallet_config: Timestamped::default(),
            referred_by,
            referrals: Referrals::default(),
            message_activity_events: MessageActivityEvents::default(),
            stable_memory_keys_to_garbage_collect: Vec::new(),
            local_user_index_event_sync_queue: BatchedTimerJobQueue::new(local_user_index_canister_id, true),
            idempotency_checker: IdempotencyChecker::default(),
            bots: InstalledBots::default(),
        }
    }

    pub fn membership(&self, now: TimestampMillis) -> Membership {
        match self.diamond_membership_expires_at {
            Some(ts) if ts > LIFETIME_DIAMOND_TIMESTAMP => Membership::LifetimeDiamond,
            Some(ts) if ts > now => Membership::Diamond,
            _ => Membership::Basic,
        }
    }

    pub fn verify_not_suspended(&self) -> Result<(), OCErrorCode> {
        if self.suspended.value { Err(OCErrorCode::InitiatorSuspended) } else { Ok(()) }
    }

    pub fn remove_group(&mut self, chat_id: ChatId, now: TimestampMillis) -> Option<GroupChat> {
        self.favourite_chats.remove(&Chat::Group(chat_id), now);
        self.hot_group_exclusions.add(chat_id, None, now);
        self.group_chats.remove(chat_id, now)
    }

    pub fn remove_community(&mut self, community_id: CommunityId, now: TimestampMillis) -> Option<Community> {
        let community = self.communities.remove(community_id, now)?;
        for channel_id in community.channels.keys() {
            self.favourite_chats.remove(&Chat::Channel(community_id, *channel_id), now);
        }
        Some(community)
    }

    pub fn handle_event_expiry(&mut self, expiry: TimestampMillis, now: TimestampMillis) {
        if self.next_event_expiry.is_none_or(|ex| expiry < ex) {
            self.next_event_expiry = Some(expiry);

            let timer_jobs = &mut self.timer_jobs;
            timer_jobs.cancel_jobs(|j| matches!(j, TimerJob::RemoveExpiredEvents(_)));
            timer_jobs.enqueue_job(TimerJob::RemoveExpiredEvents(RemoveExpiredEventsJob), expiry, now);
        }
    }

    pub fn award_achievement(&mut self, achievement: Achievement, now: TimestampMillis) -> bool {
        if self.achievements.insert(achievement) {
            let amount = achievement.chit_reward() as i32;
            self.chit_events.push(ChitEarned {
                amount,
                timestamp: now,
                reason: ChitEarnedReason::Achievement(achievement),
            });
            true
        } else {
            false
        }
    }

    pub fn push_message_activity(&mut self, event: MessageActivityEvent, now: TimestampMillis) {
        if event.user_id.is_none_or(|user_id| !self.blocked_users.contains(&user_id)) {
            self.message_activity_events.push(event, now);
        }
    }

    pub fn is_bot_permitted(&self, bot_id: &UserId, initiator: &BotInitiator, required: BotPermissions) -> bool {
        // Try to get the installed bot
        let Some(bot) = self.bots.get(bot_id) else {
            return false;
        };

        // Get the granted permissions when initiated by command or API key
        let granted = match initiator {
            BotInitiator::Command(_) => &bot.permissions,
            BotInitiator::Autonomous => match bot.autonomous_permissions.as_ref() {
                Some(permissions) => permissions,
                None => return false,
            },
        };

        // The permissions required must be a subset of the permissions granted to the bot
        required.is_subset(granted)
    }

    pub fn flush_pending_events(&mut self) {
        self.user_canister_events_queue.flush();
        self.local_user_index_event_sync_queue.flush();
    }
}

struct UserEventPusher<'a> {
    now: TimestampMillis,
    rng: &'a mut StdRng,
    queue: &'a mut BatchedTimerJobQueue<LocalUserIndexEventBatch>,
}

impl EventPusher for UserEventPusher<'_> {
    fn push(&mut self, event: Event) {
        self.queue.push(IdempotentEnvelope {
            created_at: self.now,
            idempotency_id: self.rng.next_u64(),
            value: local_user_index_canister::UserEvent::EventStoreEvent(event),
        })
    }
}

#[derive(Serialize, Debug)]
pub struct Metrics {
    pub now: TimestampMillis,
    pub heap_memory_used: u64,
    pub stable_memory_used: u64,
    pub cycles_balance: Cycles,
    pub liquid_cycles_balance: Cycles,
    pub wasm_version: BuildVersion,
    pub git_commit_id: String,
    pub direct_chats: u32,
    pub group_chats: u32,
    pub communities: u32,
    pub groups_created: u32,
    pub blocked_users: u32,
    pub created: TimestampMillis,
    pub direct_chat_metrics: ChatMetrics,
    pub video_call_operators: Vec<Principal>,
    pub timer_jobs: u32,
    pub queued_user_events: u32,
    pub queued_local_index_events: u32,
    pub chit_balance: i32,
    pub streak: u16,
    pub streak_ends: TimestampMillis,
    pub max_streak: u16,
    pub next_daily_claim: TimestampMillis,
    pub achievements: Vec<Achievement>,
    pub unique_person_proof: bool,
    pub referred_by: Option<UserId>,
    pub stable_memory_sizes: BTreeMap<u8, u64>,
    pub canister_ids: CanisterIds,
}

fn execute_update<F: FnOnce(&mut RuntimeState) -> R, R>(f: F) -> R {
    mutate_state(|state| {
        state.regular_jobs.run(state.env.deref(), &mut state.data);
        let result = f(state);
        state.data.flush_pending_events();
        result
    })
}

async fn execute_update_async<F: FnOnce() -> Fut, Fut: Future<Output = R>, R>(f: F) -> R {
    run_regular_jobs();
    let result = f().await;
    flush_pending_events();
    result
}

fn run_regular_jobs() {
    mutate_state(|state| state.regular_jobs.run(state.env.deref(), &mut state.data));
}

fn flush_pending_events() {
    mutate_state(|state| state.data.flush_pending_events());
}

#[derive(Serialize, Debug)]
pub struct CanisterIds {
    pub user_index: CanisterId,
    pub group_index: CanisterId,
    pub local_user_index: CanisterId,
    pub identity: CanisterId,
    pub escrow: CanisterId,
    pub icp_ledger: CanisterId,
}

pub enum Membership {
    Basic,
    Diamond,
    LifetimeDiamond,
}

impl Membership {
    pub fn is_diamond_member(&self) -> bool {
        matches!(self, Membership::Diamond | Membership::LifetimeDiamond)
    }

    pub fn group_creation_limit(&self) -> u32 {
        match self {
            Membership::Basic => 5,
            Membership::Diamond => 40,
            Membership::LifetimeDiamond => 100,
        }
    }
}
