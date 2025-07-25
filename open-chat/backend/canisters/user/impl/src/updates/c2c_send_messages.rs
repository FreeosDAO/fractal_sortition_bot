use crate::updates::send_message::register_timer_jobs;
use crate::{RuntimeState, UserEventPusher, execute_update_async, mutate_state, read_state};
use canister_tracing_macros::trace;
use chat_events::{MessageContentInternal, PushMessageArgs, Reader, ReplyContextInternal, ValidateNewMessageContentResult};
use ic_cdk::update;
use oc_error_codes::OCErrorCode;
use rand::Rng;
use types::{
    BotCaller, BotMessageContext, CanisterId, Chat, ContentValidationError, DirectMessageNotification, EventWrapper, FcmData,
    Message, MessageContent, MessageId, MessageIndex, SenderContext, TimestampMillis, User, UserId, UserNotificationPayload,
    UserType,
};
use user_canister::{C2CReplyContext, MessageActivity, MessageActivityEvent};

#[update]
#[trace]
async fn c2c_handle_bot_messages(
    args: user_canister::c2c_handle_bot_messages::Args,
) -> user_canister::c2c_handle_bot_messages::Response {
    execute_update_async(|| c2c_handle_bot_messages_impl(args)).await
}

async fn c2c_handle_bot_messages_impl(
    args: user_canister::c2c_handle_bot_messages::Args,
) -> user_canister::c2c_handle_bot_messages::Response {
    let (sender_status, now) = read_state(|state| (get_sender_status(state), state.env.now()));

    let (sender, sender_user_type) = match sender_status {
        SenderStatus::Ok(user_id, user_type) => (user_id, user_type),
        SenderStatus::Blocked => {
            return user_canister::c2c_handle_bot_messages::Response::Error(OCErrorCode::InitiatorBlocked.into());
        }
        SenderStatus::UnknownUser(local_user_index_canister_id, user_id) => {
            let user_type = match verify_user(local_user_index_canister_id, user_id).await {
                Some(UserType::Bot) => UserType::Bot,
                Some(UserType::OcControlledBot) => UserType::OcControlledBot,
                Some(UserType::BotV2) => UserType::BotV2,
                _ => panic!("This request is not from a bot registered with OpenChat"),
            };
            (user_id, user_type)
        }
    };

    let mut messages = Vec::new();
    for message in args.messages {
        match MessageContentInternal::validate_new_message(message.content.clone(), true, sender_user_type, false, now) {
            ValidateNewMessageContentResult::Success(content) => {
                messages.push((message, content));
            }
            ValidateNewMessageContentResult::Error(error) => {
                return user_canister::c2c_handle_bot_messages::Response::ContentValidationError(error);
            }
            _ => {
                return user_canister::c2c_handle_bot_messages::Response::ContentValidationError(
                    ContentValidationError::Unauthorized,
                );
            }
        }
    }

    mutate_state(|state| {
        let now = state.env.now();
        for (message, content) in messages {
            handle_message_impl(
                HandleMessageArgs {
                    sender,
                    thread_root_message_id: message.thread_root_message_id,
                    message_id: message.message_id,
                    sender_message_index: None,
                    sender_name: args.bot_name.clone(),
                    sender_display_name: args.bot_display_name.clone(),
                    content,
                    replies_to: None,
                    forwarding: false,
                    sender_user_type,
                    sender_avatar_id: None,
                    push_message_sent_event: true,
                    mentioned: Vec::new(),
                    mute_notification: false,
                    block_level_markdown: message.block_level_markdown.unwrap_or_default(),
                    now,
                },
                None,
                false,
                state,
            );
        }
    });
    user_canister::c2c_handle_bot_messages::Response::Success
}

pub(crate) struct HandleMessageArgs {
    pub sender: UserId,
    pub thread_root_message_id: Option<MessageId>,
    pub message_id: Option<MessageId>,
    pub sender_message_index: Option<MessageIndex>,
    pub sender_name: String,
    pub sender_display_name: Option<String>,
    pub content: MessageContentInternal,
    pub replies_to: Option<C2CReplyContext>,
    pub forwarding: bool,
    pub sender_user_type: UserType,
    pub sender_avatar_id: Option<u128>,
    pub push_message_sent_event: bool,
    pub mute_notification: bool,
    pub mentioned: Vec<User>,
    pub block_level_markdown: bool,
    pub now: TimestampMillis,
}

pub(crate) enum SenderStatus {
    Ok(UserId, UserType),
    Blocked,
    UnknownUser(CanisterId, UserId),
}

pub(crate) fn get_sender_status(state: &RuntimeState) -> SenderStatus {
    let sender = state.env.caller().into();

    if state.data.blocked_users.contains(&sender) {
        SenderStatus::Blocked
    } else if let Some(user_type) = state.data.direct_chats.get(&sender.into()).map(|c| c.user_type) {
        SenderStatus::Ok(sender, user_type)
    } else {
        SenderStatus::UnknownUser(state.data.local_user_index_canister_id, sender)
    }
}

pub(crate) async fn verify_user(local_user_index_canister_id: CanisterId, user_id: UserId) -> Option<UserType> {
    let args = local_user_index_canister::c2c_lookup_user::Args {
        user_id_or_principal: user_id.into(),
    };
    if let Ok(response) = local_user_index_canister_c2c_client::c2c_lookup_user(local_user_index_canister_id, &args).await {
        if let local_user_index_canister::c2c_lookup_user::Response::Success(r) = response {
            Some(r.user_type)
        } else {
            None
        }
    } else {
        panic!("Failed to call local_user_index to verify user");
    }
}

pub(crate) fn handle_message_impl(
    args: HandleMessageArgs,
    bot_caller: Option<BotCaller>,
    finalised: bool,
    state: &mut RuntimeState,
) -> EventWrapper<Message> {
    let chat_id = args.sender.into();
    let replies_to = convert_reply_context(args.replies_to, args.sender, state);
    let files = args.content.blob_references();

    let chat = state
        .data
        .direct_chats
        .get_or_create(args.sender, args.sender_user_type, || state.env.rng().r#gen(), args.now);

    let thread_root_message_index = args.thread_root_message_id.map(|id| chat.main_message_id_to_index(id));

    let chat_private_replying_to = if let Some((chat, None)) = replies_to.as_ref().and_then(|r| r.chat_if_other) {
        Some(chat)
    } else {
        None
    };

    let message_id = args.message_id.unwrap_or_else(|| state.env.rng().r#gen());

    let push_message_args = PushMessageArgs {
        thread_root_message_index,
        message_id,
        sender: args.sender,
        content: args.content,
        mentioned: Vec::new(),
        replies_to,
        forwarded: args.forwarding,
        sender_is_bot: args.sender_user_type.is_bot(),
        block_level_markdown: args.block_level_markdown,
        now: args.now,
        sender_context: bot_caller.map(|bot| SenderContext::Bot(BotMessageContext::from(&bot, finalised))),
    };

    let message_event = chat.push_message(
        push_message_args,
        args.sender_message_index,
        args.push_message_sent_event.then_some(UserEventPusher {
            now: args.now,
            rng: state.env.rng(),
            queue: &mut state.data.local_user_index_event_sync_queue,
        }),
    );

    let content = &message_event.event.content;

    if args.sender_user_type.is_bot() {
        chat.mark_read_up_to(message_event.event.message_index, false, args.now);
    }

    if !args.mute_notification && !chat.notifications_muted.value && !state.data.suspended.value {
        let message_type = content.content_type().to_string();
        let message_text = content.notification_text(&args.mentioned, &[]);
        let image_url = content.notification_image_url();

        let fcm_data = FcmData::for_direct_chat(args.sender)
            .set_body_with_alt(&message_text, &message_type)
            .set_optional_image(image_url.clone())
            .set_sender_name_with_alt(&args.sender_display_name, &args.sender_name)
            .set_avatar_id(args.sender_avatar_id);

        let notification = UserNotificationPayload::DirectMessage(DirectMessageNotification {
            sender: args.sender,
            thread_root_message_index,
            message_index: message_event.event.message_index,
            event_index: message_event.index,
            sender_name: args.sender_name,
            sender_display_name: args.sender_display_name,
            message_type,
            message_text,
            image_url,
            sender_avatar_id: args.sender_avatar_id,
            crypto_transfer: content.notification_crypto_transfer_details(&[]),
        });
        let recipient = state.env.canister_id().into();

        state.push_notification(Some(args.sender), recipient, notification, fcm_data);
    }

    if matches!(content, MessageContent::Crypto(_)) {
        state.data.push_message_activity(
            MessageActivityEvent {
                chat: Chat::Direct(chat_id),
                thread_root_message_index,
                message_index: message_event.event.message_index,
                message_id: message_event.event.message_id,
                event_index: message_event.index,
                activity: MessageActivity::Crypto,
                timestamp: args.now,
                user_id: Some(args.sender),
            },
            args.now,
        );
    }

    register_timer_jobs(
        chat_id,
        thread_root_message_index,
        message_id,
        &message_event,
        files,
        args.now,
        &mut state.data,
    );

    if let Some(chat) = chat_private_replying_to {
        state
            .data
            .direct_chats
            .mark_private_reply(args.sender, chat, message_event.event.message_index);
    }

    message_event
}

fn convert_reply_context(
    replies_to: Option<C2CReplyContext>,
    sender: UserId,
    state: &RuntimeState,
) -> Option<ReplyContextInternal> {
    match replies_to? {
        C2CReplyContext::ThisChat(message_id) => {
            let chat_id = sender.into();
            state
                .data
                .direct_chats
                .get(&chat_id)
                .and_then(|chat| chat.events.main_events_reader().event_index(message_id.into()))
                .map(|event_index| ReplyContextInternal {
                    chat_if_other: None,
                    event_index,
                })
        }
        C2CReplyContext::OtherChat(chat, thread_root_message_index, event_index) => Some(ReplyContextInternal {
            chat_if_other: Some((chat.into(), thread_root_message_index)),
            event_index,
        }),
    }
}
