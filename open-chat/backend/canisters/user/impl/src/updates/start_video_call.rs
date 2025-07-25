use crate::guards::caller_is_video_call_operator;
use crate::timer_job_types::{MarkVideoCallEndedJob, TimerJob};
use crate::{RuntimeState, UserEventPusher, execute_update};
use canister_tracing_macros::trace;
use chat_events::{CallParticipantInternal, MessageContentInternal, PushMessageArgs, VideoCallContentInternal};
use constants::HOUR_IN_MS;
use ic_cdk::update;
use oc_error_codes::OCErrorCode;
use rand::Rng;
use types::{
    DirectMessageNotification, EventWrapper, FcmData, Message, MessageId, MessageIndex, Milliseconds, OCResult, UserId,
    UserNotificationPayload, UserType, VideoCallPresence, VideoCallType,
};
use user_canister::start_video_call_v2::*;
use user_canister::{StartVideoCallArgs, UserCanisterEvent};

#[update(guard = "caller_is_video_call_operator")]
#[trace]
fn start_video_call_v2(args: Args) -> Response {
    execute_update(|state| start_video_call_impl(args, state)).into()
}

fn start_video_call_impl(args: Args, state: &mut RuntimeState) -> OCResult {
    let sender = args.initiator;
    let my_user_id = state.env.canister_id().into();

    if state.data.suspended.value
        || state.data.blocked_users.contains(&sender)
        || sender == my_user_id
        || matches!(args.call_type, VideoCallType::Broadcast)
    {
        return Err(OCErrorCode::InitiatorNotAuthorized.into());
    }

    let max_duration = args.max_duration.unwrap_or(HOUR_IN_MS);

    let StartVideoCallResult {
        message_event,
        mute_notification,
    } = handle_start_video_call(args.message_id, None, sender, sender, max_duration, state);

    if !mute_notification {
        // TODO i18n
        // TODO video call notifications could display decline and answer buttons
        let fcm_data = FcmData::for_direct_chat(sender)
            .set_body("Video call incoming...".to_string())
            .set_body_with_alt(&args.initiator_display_name, &args.initiator_username)
            .set_avatar_id(args.initiator_avatar_id);

        let notification = UserNotificationPayload::DirectMessage(DirectMessageNotification {
            sender,
            thread_root_message_index: None,
            message_index: message_event.event.message_index,
            event_index: message_event.index,
            sender_name: args.initiator_username,
            sender_display_name: args.initiator_display_name,
            message_type: message_event.event.content.content_type().to_string(),
            message_text: None,
            image_url: None,
            sender_avatar_id: args.initiator_avatar_id,
            crypto_transfer: None,
        });

        state.push_notification(Some(sender), my_user_id, notification, fcm_data);
    }

    state.push_user_canister_event(
        sender.into(),
        UserCanisterEvent::StartVideoCall(Box::new(StartVideoCallArgs {
            message_id: args.message_id,
            message_index: message_event.event.message_index,
            max_duration: args.max_duration,
        })),
    );

    Ok(())
}

pub fn handle_start_video_call(
    message_id: MessageId,
    their_message_index: Option<MessageIndex>,
    sender: UserId,
    other: UserId,
    max_duration: Milliseconds,
    state: &mut RuntimeState,
) -> StartVideoCallResult {
    let now = state.env.now();

    let push_message_args = PushMessageArgs {
        thread_root_message_index: None,
        message_id,
        sender,
        content: MessageContentInternal::VideoCall(VideoCallContentInternal {
            call_type: VideoCallType::Default,
            ended: None,
            participants: [(
                sender,
                CallParticipantInternal {
                    joined: now,
                    last_updated: None,
                    presence: VideoCallPresence::Owner,
                },
            )]
            .into_iter()
            .collect(),
        }),
        mentioned: Vec::new(),
        replies_to: None,
        forwarded: false,
        sender_is_bot: true,
        block_level_markdown: false,
        now,
        sender_context: None,
    };

    let chat = state
        .data
        .direct_chats
        .get_or_create(other, UserType::User, || state.env.rng().r#gen(), now);

    let mute_notification = their_message_index.is_some() || chat.notifications_muted.value;

    let message_event = chat.push_message(
        push_message_args,
        their_message_index,
        Some(UserEventPusher {
            now,
            rng: state.env.rng(),
            queue: &mut state.data.local_user_index_event_sync_queue,
        }),
    );

    if let Some(expiry) = message_event.expires_at {
        state.data.handle_event_expiry(expiry, now);
    }

    state.data.timer_jobs.enqueue_job(
        TimerJob::MarkVideoCallEnded(MarkVideoCallEndedJob(user_canister::end_video_call_v2::Args {
            user_id: other,
            message_id,
        })),
        now + max_duration,
        now,
    );

    StartVideoCallResult {
        message_event,
        mute_notification,
    }
}

pub struct StartVideoCallResult {
    pub message_event: EventWrapper<Message>,
    pub mute_notification: bool,
}
