use crate::memory::{get_instruction_counts_data_memory, get_instruction_counts_index_memory};
use crate::model::local_user_index_event_batch::LocalUserIndexEventBatch;
use crate::timer_job_types::{DeleteFileReferencesJob, MakeTransferJob, RemoveExpiredEventsJob, TimerJob};
use crate::updates::c2c_freeze_group::freeze_group_impl;
use activity_notification_state::ActivityNotificationState;
use canister_state_macros::canister_state;
use canister_timer_jobs::{Job, TimerJobs};
use chat_events::{ChatEventInternal, EventPusher, Reader};
use constants::{DAY_IN_MS, HOUR_IN_MS, ICP_LEDGER_CANISTER_ID, OPENCHAT_BOT_USER_ID};
use event_store_types::Event;
use fire_and_forget_handler::FireAndForgetHandler;
use gated_groups::{GatePayment, calculate_gate_payments};
use group_chat_core::{AddResult as AddMemberResult, GroupChatCore, GroupMemberInternal, InvitedUsersSuccess, UserInvitation};
use group_community_common::{
    Achievements, ExpiringMemberActions, ExpiringMembers, PaymentReceipts, PaymentRecipient, PendingPayment,
    PendingPaymentReason, PendingPaymentsQueue, UserCache,
};
use ic_principal::Principal;
use installed_bots::InstalledBots;
use instruction_counts_log::{InstructionCountEntry, InstructionCountFunctionId, InstructionCountsLog};
use model::user_event_batch::UserEventBatch;
use msgpack::serialize_then_unwrap;
use oc_error_codes::OCErrorCode;
use principal_to_user_id_map::PrincipalToUserIdMap;
use rand::RngCore;
use rand::prelude::StdRng;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use stable_memory_map::{BaseKeyPrefix, ChatEventKeyPrefix, StableMemoryMap};
use std::cell::RefCell;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Deref;
use timer_job_queues::{BatchedTimerJobQueue, GroupedTimerJobQueue};
use types::{
    AccessGateConfigInternal, Achievement, BotAdded, BotEventsCaller, BotInitiator, BotNotification, BotPermissions,
    BotRemoved, BotSubscriptions, BotUpdated, BuildVersion, Caller, CanisterId, ChatEventCategory, ChatId, ChatMetrics,
    CommunityId, Cycles, Document, EventIndex, EventsCaller, FcmData, FrozenGroupInfo, GroupCanisterGroupChatSummary,
    GroupMembership, GroupPermissions, GroupSubtype, IdempotentEnvelope, MAX_THREADS_IN_SUMMARY, MessageIndex, Milliseconds,
    MultiUserChat, Notification, OCResult, Rules, TimestampMillis, Timestamped, UserId, UserNotification,
    UserNotificationPayload, UserType,
};
use user_canister::GroupCanisterEvent;
use utils::env::Environment;
use utils::idempotency_checker::IdempotencyChecker;
use utils::regular_jobs::RegularJobs;

mod activity_notifications;
mod guards;
mod jobs;
mod lifecycle;
mod memory;
mod model;
mod queries;
mod regular_jobs;
mod timer_job_types;
mod updates;

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

    pub fn is_caller_video_call_operator(&self) -> bool {
        let caller = self.env.caller();
        self.data.video_call_operators.contains(&caller)
    }

    pub fn is_caller_community_being_imported_into(&self) -> bool {
        if let Some(community_id) = self
            .data
            .community_being_imported_into
            .as_ref()
            .and_then(|c| c.community_id())
        {
            CommunityId::from(self.env.caller()) == community_id
        } else {
            false
        }
    }

    pub fn get_caller_user_id(&self) -> Result<UserId, OCErrorCode> {
        let caller = self.env.caller();
        self.data
            .principal_to_user_id_map
            .get(&caller)
            .ok_or(OCErrorCode::InitiatorNotInChat)
    }

    pub fn get_calling_member(&self, verify: bool) -> Result<GroupMemberInternal, OCErrorCode> {
        let caller = self.env.caller();
        let member = self.data.get_member(caller).ok_or(OCErrorCode::InitiatorNotInChat)?;
        if verify {
            member.verify()?;
        }
        Ok(member)
    }

    pub fn push_notification(
        &mut self,
        sender: Option<UserId>,
        recipients: Vec<UserId>,
        notification: UserNotificationPayload,
        fcm_data: FcmData,
    ) {
        if !recipients.is_empty() {
            let notification = Notification::User(UserNotification {
                sender,
                recipients,
                notification_bytes: ByteBuf::from(serialize_then_unwrap(notification)),
                fcm_data: Some(fcm_data),
            });
            self.push_notification_inner(notification);
        }
    }

    pub fn push_bot_notifications(&mut self, notifications: Vec<Option<BotNotification>>) {
        for notification in notifications {
            self.push_bot_notification(notification);
        }
    }

    pub fn push_bot_notification(&mut self, notification: Option<BotNotification>) {
        if let Some(notification) = notification {
            if !notification.recipients.is_empty() {
                self.push_notification_inner(Notification::Bot(notification));
            }
        }
    }

    fn push_notification_inner(&mut self, notification: Notification) {
        self.data.local_user_index_event_sync_queue.push(IdempotentEnvelope {
            created_at: self.env.now(),
            idempotency_id: self.env.rng().next_u64(),
            value: local_user_index_canister::GroupEvent::Notification(Box::new(notification)),
        });
    }

    pub fn queue_access_gate_payments(&mut self, payment: GatePayment) {
        for payment in calculate_gate_payments(payment, self.data.chat.members.owners()) {
            self.data.pending_payments_queue.push(payment);
        }

        jobs::make_pending_payments::start_job_if_required(self);
    }

    pub fn summary(&self, member: &GroupMemberInternal) -> GroupCanisterGroupChatSummary {
        let chat = &self.data.chat;
        let min_visible_event_index = member.min_visible_event_index();
        let min_visible_message_index = member.min_visible_message_index();
        let main_events_reader = chat.events.visible_main_events_reader(min_visible_event_index);
        let events_ttl = chat.events.get_events_time_to_live();

        let membership = GroupMembership {
            joined: member.date_added(),
            role: member.role().value.into(),
            mentions: chat.most_recent_mentions(member, None),
            notifications_muted: member.notifications_muted().value,
            my_metrics: chat
                .events
                .user_metrics(&member.user_id(), None)
                .map(|m| m.hydrate())
                .unwrap_or_default(),
            latest_threads: member
                .followed_threads
                .iter()
                .rev()
                .filter_map(|(i, _)| self.data.chat.events.thread_details(i))
                .take(MAX_THREADS_IN_SUMMARY)
                .collect(),
            rules_accepted: member
                .rules_accepted
                .as_ref()
                .is_some_and(|version| version.value >= chat.rules.text.version),
            lapsed: member.lapsed().value,
        };

        GroupCanisterGroupChatSummary {
            chat_id: self.env.canister_id().into(),
            local_user_index_canister_id: self.data.local_user_index_canister_id,
            last_updated: chat.last_updated(Some(member.user_id())),
            name: chat.name.value.clone(),
            description: chat.description.value.clone(),
            subtype: chat.subtype.value.clone(),
            avatar_id: Document::id(&chat.avatar),
            is_public: chat.is_public.value,
            history_visible_to_new_joiners: chat.history_visible_to_new_joiners,
            messages_visible_to_non_members: chat.messages_visible_to_non_members.value,
            min_visible_event_index,
            min_visible_message_index,
            latest_message: main_events_reader.latest_message_event(Some(member.user_id())),
            latest_event_index: main_events_reader.latest_event_index().unwrap_or_default(),
            latest_message_index: main_events_reader.latest_message_index(),
            joined: membership.joined,
            participant_count: chat.members.len(),
            role: membership.role,
            mentions: membership.mentions.clone(),
            permissions_v2: chat.permissions.value.clone(),
            notifications_muted: membership.notifications_muted,
            metrics: chat.events.metrics().hydrate(),
            my_metrics: membership.my_metrics.clone(),
            latest_threads: membership.latest_threads.clone(),
            frozen: self.data.frozen.value.clone(),
            wasm_version: BuildVersion::default(),
            date_last_pinned: chat.date_last_pinned,
            events_ttl: events_ttl.value,
            events_ttl_last_updated: events_ttl.timestamp,
            gate_config: chat.gate_config.value.clone().map(|gc| gc.into()),
            rules_accepted: membership.rules_accepted,
            membership: Some(membership),
            video_call_in_progress: chat.events.video_call_in_progress(Some(member.user_id())),
            verified: self.data.verified.value,
        }
    }

    pub fn add_member(&mut self, args: AddMemberArgs) -> AddMemberResult {
        let result = self.data.chat.members.add(
            args.user_id,
            args.now,
            args.min_visible_event_index,
            args.min_visible_message_index,
            args.mute_notifications,
            args.user_type,
        );

        if matches!(result, AddMemberResult::Success(_) | AddMemberResult::AlreadyInGroup) {
            self.data.principal_to_user_id_map.insert(args.principal, args.user_id);
        }

        result
    }

    pub fn start_importing_into_community(
        &mut self,
        community: CommunityBeingImportedInto,
    ) -> OCResult<StartImportIntoCommunityResultSuccess> {
        if self.data.community_being_imported_into.is_some() && self.data.is_frozen() {
            Err(OCErrorCode::AlreadyImportingIntoAnotherCommunity.into())
        } else if self.data.is_frozen() {
            Err(OCErrorCode::ChatFrozen.into())
        } else {
            let transfers_required = self.prepare_transfers_for_import_into_community();
            let serialized = serialize_then_unwrap(&self.data.chat);
            let total_bytes = serialized.len() as u64;

            if let Some(community_id) = community.community_id() {
                self.transfer_funds_to_community_being_imported_into(community_id, &transfers_required);
            }

            self.data.community_being_imported_into = Some(community);
            self.data.serialized_chat_state = Some(ByteBuf::from(serialized));

            freeze_group_impl(
                OPENCHAT_BOT_USER_ID,
                Some("Chat is being imported into a community".to_string()),
                false,
                self,
            );

            Ok(StartImportIntoCommunityResultSuccess {
                total_bytes,
                transfers_required,
            })
        }
    }

    pub fn prepare_transfers_for_import_into_community(&mut self) -> HashMap<CanisterId, (u128, u128)> {
        let now = self.env.now();
        let max_prize_message_length = 7 * DAY_IN_MS;
        let pending_prize_messages = self
            .data
            .chat
            .events
            .pending_prize_messages(now.saturating_sub(max_prize_message_length));

        let mut transfers_required = HashMap::new();

        for (message_id, prize_message) in pending_prize_messages {
            let ledger = prize_message.transaction.ledger_canister_id();
            let fee = prize_message.transaction.fee();
            let amount: u128 = prize_message.prizes_remaining.iter().map(|p| p + fee).sum();

            match transfers_required.entry(ledger) {
                Vacant(e) => {
                    e.insert((amount.saturating_sub(fee), fee));
                    self.data.chat.events.reduce_final_prize_by_transfer_fee(message_id, now);
                }
                Occupied(e) => {
                    let (total, _) = e.into_mut();
                    *total += amount;
                }
            }
        }

        transfers_required
    }

    fn transfer_funds_to_community_being_imported_into(
        &mut self,
        community_id: CommunityId,
        transfers: &HashMap<CanisterId, (u128, u128)>,
    ) {
        for (&ledger_canister, &(amount, fee)) in transfers.iter() {
            self.data.pending_payments_queue.push(PendingPayment {
                amount,
                fee,
                ledger_canister,
                recipient: PaymentRecipient::Account(Principal::from(community_id).into()),
                reason: PendingPaymentReason::TransferToCommunityBeingImportedInto,
            });
        }
        jobs::make_pending_payments::start_job_if_required(self);
    }

    pub fn run_event_expiry_job(&mut self) {
        let now = self.env.now();
        let result = self.data.chat.remove_expired_events(now);

        self.data.next_event_expiry = self.data.chat.events.next_event_expiry();
        if let Some(expiry) = self.data.next_event_expiry {
            self.data
                .timer_jobs
                .enqueue_job(TimerJob::RemoveExpiredEvents(RemoveExpiredEventsJob), expiry, now);
        }
        if !result.files.is_empty() {
            let delete_files_job = DeleteFileReferencesJob { files: result.files };
            delete_files_job.execute();
        }
        for pending_transaction in result.final_prize_payments {
            self.data.timer_jobs.enqueue_job(
                TimerJob::MakeTransfer(Box::new(MakeTransferJob {
                    pending_transaction,
                    attempt: 0,
                })),
                now,
                now,
            );
        }
        for thread in result.threads {
            self.data
                .stable_memory_keys_to_garbage_collect
                .push(BaseKeyPrefix::from(ChatEventKeyPrefix::new_from_group_chat(Some(
                    thread.root_message_index,
                ))));
        }
        jobs::garbage_collect_stable_memory::start_job_if_required(self);
    }

    pub fn push_event_to_user(&mut self, user_id: UserId, event: GroupCanisterEvent, now: TimestampMillis) {
        self.data.user_event_sync_queue.push(
            user_id,
            IdempotentEnvelope {
                created_at: now,
                idempotency_id: self.env.rng().next_u64(),
                value: event,
            },
        );
    }

    pub fn notify_user_of_achievement(&mut self, user_id: UserId, achievement: Achievement, now: TimestampMillis) {
        if !self.data.chat.members.bots().contains_key(&user_id) && self.data.achievements.award(user_id, achievement).is_some()
        {
            self.push_event_to_user(user_id, GroupCanisterEvent::Achievement(achievement), now);
        }
    }

    pub fn metrics(&self) -> Metrics {
        let group_chat_core = &self.data.chat;
        let now = self.env.now();
        let messages_in_last_hour = group_chat_core
            .events
            .event_count_since(now.saturating_sub(HOUR_IN_MS), |e| e.is_message()) as u64;
        let messages_in_last_day = group_chat_core
            .events
            .event_count_since(now.saturating_sub(DAY_IN_MS), |e| e.is_message()) as u64;
        let events_in_last_hour = group_chat_core
            .events
            .event_count_since(now.saturating_sub(HOUR_IN_MS), |_| true) as u64;
        let events_in_last_day = group_chat_core
            .events
            .event_count_since(now.saturating_sub(DAY_IN_MS), |_| true) as u64;

        Metrics {
            heap_memory_used: utils::memory::heap(),
            stable_memory_used: utils::memory::stable(),
            now,
            cycles_balance: self.env.cycles_balance(),
            liquid_cycles_balance: self.env.liquid_cycles_balance(),
            wasm_version: WASM_VERSION.with_borrow(|v| **v),
            git_commit_id: utils::git::git_commit_id().to_string(),
            public: group_chat_core.is_public.value,
            date_created: group_chat_core.date_created,
            members: group_chat_core.members.len(),
            moderators: group_chat_core.members.moderators().len() as u32,
            admins: group_chat_core.members.admins().len() as u32,
            owners: group_chat_core.members.owners().len() as u32,
            blocked: group_chat_core.members.blocked().len() as u32,
            invited: self.data.chat.invited_users.len() as u32,
            chat_metrics: group_chat_core.events.metrics().hydrate(),
            messages_in_last_hour,
            messages_in_last_day,
            events_in_last_hour,
            events_in_last_day,
            frozen_at: self.data.frozen.as_ref().map(|f| f.timestamp),
            instruction_counts: self.data.instruction_counts_log.iter().collect(),
            community_being_imported_into: self
                .data
                .community_being_imported_into
                .as_ref()
                .and_then(|c| c.community_id()),
            serialized_chat_state_bytes: self
                .data
                .serialized_chat_state
                .as_ref()
                .map(|bytes| bytes.len() as u64)
                .unwrap_or_default(),
            timer_jobs: self.data.timer_jobs.len() as u32,
            queued_user_events: self.data.user_event_sync_queue.len() as u32,
            queued_local_index_events: self.data.local_user_index_event_sync_queue.len() as u32,
            stable_memory_sizes: memory::memory_sizes(),
            canister_ids: CanisterIds {
                user_index: self.data.user_index_canister_id,
                group_index: self.data.group_index_canister_id,
                local_user_index: self.data.local_user_index_canister_id,
                proposals_bot: self.data.proposals_bot_user_id.into(),
                escrow_canister_id: self.data.escrow_canister_id,
                icp_ledger: ICP_LEDGER_CANISTER_ID,
            },
        }
    }

    pub fn verified_caller(&self, ext_caller: Option<Caller>) -> OCResult<Caller> {
        match ext_caller {
            Some(Caller::BotV2(bot)) => return Ok(Caller::BotV2(bot)),
            Some(Caller::Webhook(user_id)) => return Ok(Caller::Webhook(user_id)),
            _ => {}
        }

        let caller = self.env.caller();

        if caller == self.data.user_index_canister_id {
            return Ok(Caller::OCBot(OPENCHAT_BOT_USER_ID));
        }

        let Some(user_id) = self.data.lookup_user_id(caller) else {
            return Err(OCErrorCode::InitiatorNotFound.into());
        };

        let member = self.data.chat.members.get_verified_member(user_id)?;

        match member.user_type() {
            UserType::User => Ok(Caller::User(member.user_id())),
            UserType::Bot => Ok(Caller::Bot(member.user_id())),
            UserType::OcControlledBot => Ok(Caller::OCBot(member.user_id())),
            UserType::BotV2 | UserType::Webhook => Err(OCErrorCode::InitiatorNotFound.into()),
        }
    }

    pub fn mark_activity_for_user(&mut self, user_id: UserId) {
        let now = self.env.now();

        self.data.local_user_index_event_sync_queue.push(IdempotentEnvelope {
            created_at: now,
            idempotency_id: self.env.rng().next_u64(),
            value: local_user_index_canister::GroupEvent::MarkActivityForUser(now, user_id),
        });
    }
}

#[derive(Serialize, Deserialize)]
struct Data {
    pub chat: GroupChatCore,
    pub principal_to_user_id_map: PrincipalToUserIdMap,
    pub group_index_canister_id: CanisterId,
    pub user_index_canister_id: CanisterId,
    pub local_user_index_canister_id: CanisterId,
    pub proposals_bot_user_id: UserId,
    pub escrow_canister_id: CanisterId,
    pub internet_identity_canister_id: CanisterId,
    pub invite_code: Option<u64>,
    pub invite_code_enabled: bool,
    pub frozen: Timestamped<Option<FrozenGroupInfo>>,
    pub timer_jobs: TimerJobs<TimerJob>,
    pub fire_and_forget_handler: FireAndForgetHandler,
    pub activity_notification_state: ActivityNotificationState,
    #[serde(skip, default = "init_instruction_counts_log")]
    pub instruction_counts_log: InstructionCountsLog,
    pub test_mode: bool,
    pub community_being_imported_into: Option<CommunityBeingImportedInto>,
    pub serialized_chat_state: Option<ByteBuf>,
    pub next_event_expiry: Option<TimestampMillis>,
    pub rng_seed: [u8; 32],
    pub pending_payments_queue: PendingPaymentsQueue,
    pub total_payment_receipts: PaymentReceipts,
    pub video_call_operators: Vec<Principal>,
    #[serde(with = "serde_bytes")]
    pub ic_root_key: Vec<u8>,
    achievements: Achievements,
    expiring_members: ExpiringMembers,
    expiring_member_actions: ExpiringMemberActions,
    user_cache: UserCache,
    user_event_sync_queue: GroupedTimerJobQueue<UserEventBatch>,
    local_user_index_event_sync_queue: BatchedTimerJobQueue<LocalUserIndexEventBatch>,
    stable_memory_keys_to_garbage_collect: Vec<BaseKeyPrefix>,
    verified: Timestamped<bool>,
    pub bots: InstalledBots,
    idempotency_checker: IdempotencyChecker,
}

fn init_instruction_counts_log() -> InstructionCountsLog {
    InstructionCountsLog::init(get_instruction_counts_index_memory(), get_instruction_counts_data_memory())
}

#[expect(clippy::too_many_arguments)]
impl Data {
    pub fn new(
        chat_id: ChatId,
        is_public: bool,
        name: String,
        description: String,
        rules: Rules,
        subtype: Option<GroupSubtype>,
        avatar: Option<Document>,
        history_visible_to_new_joiners: bool,
        messages_visible_to_non_members: bool,
        creator_principal: Principal,
        creator_user_id: UserId,
        creator_user_type: UserType,
        events_ttl: Option<Milliseconds>,
        now: TimestampMillis,
        mark_active_duration: Milliseconds,
        group_index_canister_id: CanisterId,
        user_index_canister_id: CanisterId,
        local_user_index_canister_id: CanisterId,
        proposals_bot_user_id: UserId,
        escrow_canister_id: CanisterId,
        internet_identity_canister_id: CanisterId,
        test_mode: bool,
        permissions: Option<GroupPermissions>,
        gate_config: Option<AccessGateConfigInternal>,
        video_call_operators: Vec<Principal>,
        ic_root_key: Vec<u8>,
        anonymized_chat_id: u128,
    ) -> Data {
        let chat = GroupChatCore::new(
            MultiUserChat::Group(chat_id),
            creator_user_id,
            is_public,
            name,
            description,
            rules,
            subtype,
            avatar,
            history_visible_to_new_joiners,
            messages_visible_to_non_members,
            permissions.unwrap_or_default(),
            gate_config,
            events_ttl,
            creator_user_type,
            anonymized_chat_id,
            None,
            now,
        );

        let mut principal_to_user_id_map = PrincipalToUserIdMap::default();
        principal_to_user_id_map.insert(creator_principal, creator_user_id);

        Data {
            chat,
            principal_to_user_id_map,
            group_index_canister_id,
            user_index_canister_id,
            local_user_index_canister_id,
            proposals_bot_user_id,
            escrow_canister_id,
            internet_identity_canister_id,
            activity_notification_state: ActivityNotificationState::new(now, mark_active_duration),
            test_mode,
            invite_code: None,
            invite_code_enabled: false,
            frozen: Timestamped::default(),
            timer_jobs: TimerJobs::default(),
            fire_and_forget_handler: FireAndForgetHandler::default(),
            instruction_counts_log: init_instruction_counts_log(),
            community_being_imported_into: None,
            serialized_chat_state: None,
            next_event_expiry: None,
            rng_seed: [0; 32],
            pending_payments_queue: PendingPaymentsQueue::default(),
            total_payment_receipts: PaymentReceipts::default(),
            video_call_operators,
            ic_root_key,
            achievements: Achievements::default(),
            expiring_members: ExpiringMembers::default(),
            expiring_member_actions: ExpiringMemberActions::default(),
            user_cache: UserCache::default(),
            user_event_sync_queue: GroupedTimerJobQueue::new(5, true),
            local_user_index_event_sync_queue: BatchedTimerJobQueue::new(local_user_index_canister_id, true),
            stable_memory_keys_to_garbage_collect: Vec::new(),
            verified: Timestamped::default(),
            bots: InstalledBots::default(),
            idempotency_checker: IdempotencyChecker::default(),
        }
    }

    pub fn lookup_user_id(&self, user_id_or_principal: Principal) -> Option<UserId> {
        let user_id = self
            .principal_to_user_id_map
            .get(&user_id_or_principal)
            .unwrap_or(user_id_or_principal.into());

        self.chat.members.contains(&user_id).then_some(user_id)
    }

    pub fn get_member(&self, user_id_or_principal: Principal) -> Option<GroupMemberInternal> {
        let user_id = self
            .principal_to_user_id_map
            .get(&user_id_or_principal)
            .unwrap_or(user_id_or_principal.into());

        self.chat.members.get(&user_id)
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen.is_some()
    }

    pub fn verify_not_frozen(&self) -> Result<(), OCErrorCode> {
        if self.is_frozen() { Err(OCErrorCode::ChatFrozen) } else { Ok(()) }
    }

    pub fn is_accessible(&self, caller: Principal, invite_code: Option<u64>) -> bool {
        self.chat.is_public.value
            || self.get_member(caller).is_some()
            || self.get_invitation(caller).is_some()
            || self.is_invite_code_valid(invite_code)
    }

    pub fn verify_is_accessible(&self, caller: Principal, invite_code: Option<u64>) -> Result<(), OCErrorCode> {
        if self.is_accessible(caller, invite_code) { Ok(()) } else { Err(OCErrorCode::InitiatorNotInChat) }
    }

    pub fn get_invitation(&self, caller: Principal) -> Option<&UserInvitation> {
        self.principal_to_user_id_map
            .get(&caller)
            .and_then(|user_id| self.chat.invited_users.get(&user_id))
    }

    pub fn invite_users(
        &mut self,
        invited_by: Caller,
        users: Vec<(UserId, Principal)>,
        now: TimestampMillis,
    ) -> OCResult<InvitedUsersSuccess> {
        let user_ids: Vec<UserId> = users.iter().map(|(user_id, _)| *user_id).collect();
        let result = self.chat.invite_users(invited_by, user_ids, now)?;

        let invited_users: HashSet<UserId> = result.invited_users.iter().copied().collect();
        for (user_id, principal) in users.into_iter().filter(|(user_id, _)| invited_users.contains(user_id)) {
            self.principal_to_user_id_map.insert(principal, user_id);
        }

        Ok(result)
    }

    pub fn remove_invitation(&mut self, caller: Principal, now: TimestampMillis) -> Option<UserInvitation> {
        self.principal_to_user_id_map
            .remove(&caller)
            .map(|v| v.into_value())
            .and_then(|user_id| self.chat.invited_users.remove(&user_id, now))
    }

    pub fn record_instructions_count(&self, function_id: InstructionCountFunctionId, now: TimestampMillis) {
        let wasm_version = WASM_VERSION.with_borrow(|v| **v);
        let instructions_count = ic_cdk::api::instruction_counter();

        let _ = self
            .instruction_counts_log
            .record(function_id, instructions_count, wasm_version, now);
    }

    pub fn handle_event_expiry(&mut self, expiry: TimestampMillis, now: TimestampMillis) {
        if self.next_event_expiry.is_none_or(|ex| expiry < ex) {
            self.next_event_expiry = Some(expiry);

            let timer_jobs = &mut self.timer_jobs;
            timer_jobs.cancel_jobs(|j| matches!(j, TimerJob::RemoveExpiredEvents(_)));
            timer_jobs.enqueue_job(TimerJob::RemoveExpiredEvents(RemoveExpiredEventsJob), expiry, now);
        }
    }

    fn is_invite_code_valid(&self, invite_code: Option<u64>) -> bool {
        if self.invite_code_enabled {
            if let Some(provided_code) = invite_code {
                if let Some(stored_code) = self.invite_code {
                    return provided_code == stored_code;
                }
            }
        }

        false
    }

    pub fn remove_user(&mut self, user_id: UserId, principal: Option<Principal>) {
        if let Some(principal) = principal {
            let user_id_removed = self.principal_to_user_id_map.remove(&principal).map(|v| v.into_value());
            assert_eq!(user_id_removed, Some(user_id));
        }

        self.expiring_members.remove_member(user_id, None);
        self.expiring_member_actions.remove_member(user_id, None);
        self.achievements.remove_user(&user_id);
        self.user_cache.delete(user_id);
    }

    pub fn get_caller_for_events(&self, caller: Principal, bot_initiator: Option<BotInitiator>) -> Option<EventsCaller> {
        if let Some(initiator) = bot_initiator {
            let bot_user_id = caller.into();
            let permissions = self.granted_bot_permissions(&bot_user_id, &initiator)?;
            let bot_permitted_event_types = permissions.permitted_chat_event_categories_to_read();
            if bot_permitted_event_types.is_empty() {
                return None;
            }
            Some(EventsCaller::Bot(BotEventsCaller {
                bot: bot_user_id,
                bot_permitted_event_categories: bot_permitted_event_types,
                min_visible_event_index: EventIndex::default(),
            }))
        } else if let Some(user_id) = self.lookup_user_id(caller) {
            Some(EventsCaller::User(user_id))
        } else {
            Some(EventsCaller::Unknown)
        }
    }

    pub fn get_bot_permissions(&self, bot_user_id: &UserId) -> Option<&BotPermissions> {
        self.bots.get(bot_user_id).map(|b| &b.permissions)
    }

    pub fn get_user_permissions(&self, user_id: &UserId) -> Option<BotPermissions> {
        let member = self.chat.members.get_verified_member(*user_id).ok()?;

        let group_permissions = member.role().chat_permissions(&self.chat.permissions);
        let message_permissions = member.role().message_permissions(&self.chat.permissions.message_permissions);

        Some(
            BotPermissions::default()
                .with_chat(&group_permissions)
                .with_message(&message_permissions),
        )
    }

    pub fn is_bot_permitted(&self, bot_id: &UserId, initiator: &BotInitiator, required: &BotPermissions) -> bool {
        self.granted_bot_permissions(bot_id, initiator)
            .is_some_and(|granted| required.is_subset(&granted))
    }

    fn granted_bot_permissions(&self, bot_id: &UserId, initiator: &BotInitiator) -> Option<BotPermissions> {
        // Try to get the installed bot
        let bot = self.bots.get(bot_id)?;

        // Get the granted permissions when initiated by command or API key
        match initiator {
            BotInitiator::Command(command) => self
                .get_user_permissions(&command.initiator)
                .map(|u| BotPermissions::intersect(&bot.permissions, &u)),
            BotInitiator::Autonomous => bot.autonomous_permissions.clone(),
        }
    }

    pub fn install_bot(
        &mut self,
        owner_id: UserId,
        bot_id: UserId,
        permissions: BotPermissions,
        autonomous_permissions: Option<BotPermissions>,
        default_subscriptions: Option<BotSubscriptions>,
        now: TimestampMillis,
    ) -> bool {
        if !self.bots.add(
            bot_id,
            owner_id,
            permissions,
            autonomous_permissions.clone(),
            default_subscriptions.clone(),
            now,
        ) {
            return false;
        }

        self.chat.events.push_main_event(
            ChatEventInternal::BotAdded(Box::new(BotAdded {
                user_id: bot_id,
                added_by: owner_id,
            })),
            now,
        );

        // Subscribe to permitted chat events
        if let (Some(subscriptions), Some(permissions)) = (default_subscriptions, autonomous_permissions) {
            let permitted_categories = permissions.permitted_chat_event_categories_to_read();

            self.chat.events.subscribe_bot_to_events(
                bot_id,
                subscriptions
                    .chat
                    .into_iter()
                    .filter(|t| permitted_categories.contains(&ChatEventCategory::from(*t)))
                    .collect(),
            );
        }

        true
    }

    pub fn update_bot(
        &mut self,
        owner_id: UserId,
        bot_id: UserId,
        permissions: BotPermissions,
        autonomous_permissions: Option<BotPermissions>,
        now: TimestampMillis,
    ) -> bool {
        if !self.bots.update(bot_id, permissions, autonomous_permissions.clone(), now) {
            return false;
        }

        self.chat.events.push_main_event(
            ChatEventInternal::BotUpdated(Box::new(BotUpdated {
                user_id: bot_id,
                updated_by: owner_id,
            })),
            now,
        );

        // Subscribe to permitted chat events
        let bot = self.bots.get(&bot_id).unwrap();
        let permissions = autonomous_permissions.unwrap_or_default();
        let permitted_categories = permissions.permitted_chat_event_categories_to_read();
        let subscriptions = bot.default_subscriptions.clone().unwrap_or_default();

        self.chat.events.subscribe_bot_to_events(
            bot_id,
            subscriptions
                .chat
                .into_iter()
                .filter(|t| permitted_categories.contains(&ChatEventCategory::from(*t)))
                .collect(),
        );

        true
    }

    pub fn uninstall_bot(&mut self, owner_id: UserId, bot_id: UserId, now: TimestampMillis) -> bool {
        if !self.bots.remove(bot_id, now) {
            return false;
        }

        self.chat.events.unsubscribe_bot_from_events(bot_id);

        self.chat.events.push_main_event(
            ChatEventInternal::BotRemoved(Box::new(BotRemoved {
                user_id: bot_id,
                removed_by: owner_id,
            })),
            now,
        );

        true
    }

    pub fn details_last_updated(&self) -> TimestampMillis {
        let timestamps = vec![self.chat.details_last_updated(), self.bots.last_updated()];

        timestamps.into_iter().max().unwrap_or_default()
    }

    pub fn flush_pending_events(&mut self) {
        self.user_event_sync_queue.flush();
        self.local_user_index_event_sync_queue.flush();
    }
}

struct GroupEventPusher<'a> {
    now: TimestampMillis,
    rng: &'a mut StdRng,
    queue: &'a mut BatchedTimerJobQueue<LocalUserIndexEventBatch>,
}

impl EventPusher for GroupEventPusher<'_> {
    fn push(&mut self, event: Event) {
        self.queue.push(IdempotentEnvelope {
            created_at: self.now,
            idempotency_id: self.rng.next_u64(),
            value: local_user_index_canister::GroupEvent::EventStoreEvent(event),
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
    pub public: bool,
    pub date_created: TimestampMillis,
    pub members: u32,
    pub moderators: u32,
    pub admins: u32,
    pub owners: u32,
    pub blocked: u32,
    pub invited: u32,
    pub chat_metrics: ChatMetrics,
    pub messages_in_last_hour: u64,
    pub messages_in_last_day: u64,
    pub events_in_last_hour: u64,
    pub events_in_last_day: u64,
    pub frozen_at: Option<TimestampMillis>,
    pub instruction_counts: Vec<InstructionCountEntry>,
    pub community_being_imported_into: Option<CommunityId>,
    pub serialized_chat_state_bytes: u64,
    pub timer_jobs: u32,
    pub queued_user_events: u32,
    pub queued_local_index_events: u32,
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

struct AddMemberArgs {
    user_id: UserId,
    principal: Principal,
    now: TimestampMillis,
    min_visible_event_index: EventIndex,
    min_visible_message_index: MessageIndex,
    mute_notifications: bool,
    user_type: UserType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CommunityBeingImportedInto {
    New,
    Existing(CommunityId),
}

impl CommunityBeingImportedInto {
    fn community_id(&self) -> Option<CommunityId> {
        if let CommunityBeingImportedInto::Existing(community_id) = self {
            Some(*community_id)
        } else {
            None
        }
    }
}

#[derive(Serialize, Debug)]
pub struct CanisterIds {
    pub user_index: CanisterId,
    pub group_index: CanisterId,
    pub local_user_index: CanisterId,
    pub proposals_bot: CanisterId,
    pub escrow_canister_id: CanisterId,
    pub icp_ledger: CanisterId,
}

pub struct StartImportIntoCommunityResultSuccess {
    pub total_bytes: u64,
    pub transfers_required: HashMap<CanisterId, (u128, u128)>,
}

pub enum CallerResult {
    Success(Caller),
    NotFound,
    Lapsed,
    Suspended,
}
