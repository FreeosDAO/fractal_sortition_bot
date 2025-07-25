use super::c2c_send_messages::{HandleMessageArgs, handle_message_impl};
use crate::crypto::process_transaction_without_caller_check;
use crate::guards::{caller_is_local_user_index, caller_is_owner};
use crate::timer_job_types::{DeleteFileReferencesJob, MarkP2PSwapExpiredJob, NotifyEscrowCanisterOfDepositJob};
use crate::updates::send_message_with_transfer::set_up_p2p_swap;
use crate::{Data, RuntimeState, TimerJob, UserEventPusher, execute_update, execute_update_async, mutate_state, read_state};
use candid::Principal;
use canister_api_macros::update;
use canister_tracing_macros::trace;
use chat_events::{
    EditMessageArgs, EditMessageSuccess, MessageContentInternal, PushMessageArgs, Reader, ReplyContextInternal,
    TextContentInternal, ValidateNewMessageContentResult,
};
use constants::{MEMO_MESSAGE, OPENCHAT_BOT_USER_ID};
use oc_error_codes::OCErrorCode;
use rand::Rng;
use std::ops::Not;
use types::{
    BlobReference, BotCaller, BotPermissions, CanisterId, Chat, ChatId, CompletedCryptoTransaction, CryptoTransaction,
    DirectMessageNotification, EventIndex, EventWrapper, FcmData, Message, MessageContent, MessageContentInitial, MessageId,
    MessageIndex, OCResult, P2PSwapLocation, ReplyContext, TimestampMillis, UserId, UserNotificationPayload, UserType,
};
use user_canister::send_message_v2::{Response::*, *};
use user_canister::{C2CReplyContext, SendMessageArgs, SendMessagesArgs, UserCanisterEvent, c2c_bot_send_message};

#[update(guard = "caller_is_owner", msgpack = true)]
#[trace]
async fn send_message_v2(args: Args) -> Response {
    execute_update_async(|| send_message_v2_impl(args)).await
}

async fn send_message_v2_impl(args: Args) -> Response {
    let PrepareOk {
        my_user_id,
        now,
        local_user_index_canister_id,
        maybe_recipient_type,
    } = match read_state(|state| prepare(&args, false, state)) {
        Ok(ok) => ok,
        Err(error) => return Error(error),
    };

    let recipient_type = if let Some(recipient_type) = maybe_recipient_type {
        recipient_type
    } else {
        let c2c_args = local_user_index_canister::c2c_lookup_user::Args {
            user_id_or_principal: args.recipient.into(),
        };
        match local_user_index_canister_c2c_client::c2c_lookup_user(local_user_index_canister_id, &c2c_args).await {
            Ok(local_user_index_canister::c2c_lookup_user::Response::Success(result)) => RecipientType::Other(result.user_type),
            Ok(local_user_index_canister::c2c_lookup_user::Response::UserNotFound) => {
                return Error(OCErrorCode::TargetUserNotFound.into());
            }
            Err(error) => return Error(error.into()),
        }
    };

    let (content, completed_transfer) =
        match MessageContentInternal::validate_new_message(args.content, true, UserType::User, args.forwarding, now) {
            ValidateNewMessageContentResult::Success(content) => (content, None),
            ValidateNewMessageContentResult::SuccessCrypto(content) => {
                let mut pending_transfer = match &content.transfer {
                    CryptoTransaction::Pending(t) => t.clone().set_memo(&MEMO_MESSAGE),
                    _ => unreachable!(),
                };

                if !pending_transfer.validate_recipient(args.recipient) {
                    return Error(OCErrorCode::InvalidRequest.with_message("Transaction is not to the user's account"));
                }

                if let Err(error) = mutate_state(|state| state.data.pin_number.verify(args.pin.as_deref(), now)) {
                    return Error(error.into());
                }

                // When transferring to bot users, each user transfers to their own subaccount, this way it
                // is trivial for the bots to keep track of each user's funds
                if recipient_type.user_type().is_bot() {
                    pending_transfer.set_recipient(args.recipient.into(), Principal::from(my_user_id).into());
                }

                // We have to use `process_transaction_without_caller_check` because we may be within a
                // reply callback due to calling `c2c_lookup_user` earlier.
                match process_transaction_without_caller_check(pending_transfer).await {
                    Ok(Ok(completed)) => read_state(|state| {
                        let content = MessageContentInternal::new_with_transfer(
                            MessageContentInitial::Crypto(content),
                            completed.clone().into(),
                            None,
                            state.env.now(),
                        );
                        (content, Some(completed))
                    }),
                    Ok(Err(failed)) => return Error(OCErrorCode::TransferFailed.with_message(failed.error_message())),
                    Err(error) => return Error(error.into()),
                }
            }
            ValidateNewMessageContentResult::SuccessPrize(_) => unreachable!(),
            ValidateNewMessageContentResult::SuccessP2PSwap(content) => {
                let (escrow_canister_id, now) = read_state(|state| (state.data.escrow_canister_id, state.env.now()));
                let create_swap_args = escrow_canister::create_swap::Args {
                    location: P2PSwapLocation::from_message(Chat::Direct(args.recipient.into()), None, args.message_id),
                    token0: content.token0.clone(),
                    token0_amount: content.token0_amount,
                    token1: content.token1.clone(),
                    token1_amount: content.token1_amount,
                    expires_at: now + content.expires_in,
                    additional_admins: Vec::new(),
                    canister_to_notify: Some(args.recipient.into()),
                };
                match set_up_p2p_swap(escrow_canister_id, create_swap_args).await {
                    Ok((swap_id, pending_transaction)) => {
                        match process_transaction_without_caller_check(pending_transaction).await {
                            Ok(Ok(completed)) => {
                                NotifyEscrowCanisterOfDepositJob::run(swap_id);
                                let content = MessageContentInternal::new_with_transfer(
                                    MessageContentInitial::P2PSwap(content),
                                    completed.clone().into(),
                                    Some(swap_id),
                                    read_state(|state| state.env.now()),
                                );
                                (content, Some(completed))
                            }
                            Ok(Err(failed)) => {
                                return Error(OCErrorCode::TransferFailed.with_message(failed.error_message()));
                            }
                            Err(error) => return Error(error.into()),
                        }
                    }
                    Err(error) => return Error(error.into()),
                }
            }
            ValidateNewMessageContentResult::Error(error) => {
                return Error(OCErrorCode::InvalidMessageContent.with_json(&error));
            }
        };

    mutate_state(|state| {
        send_message_impl(
            my_user_id,
            args.recipient,
            args.thread_root_message_index,
            args.message_id,
            content,
            args.replies_to,
            args.forwarding,
            args.block_level_markdown,
            args.message_filter_failed,
            recipient_type,
            completed_transfer,
            state,
        )
    })
}

#[update(guard = "caller_is_local_user_index", msgpack = true)]
#[trace]
fn c2c_bot_send_message(args: c2c_bot_send_message::Args) -> c2c_bot_send_message::Response {
    execute_update(|state| c2c_bot_send_message_impl(args, state))
}

fn c2c_bot_send_message_impl(args: c2c_bot_send_message::Args, state: &mut RuntimeState) -> c2c_bot_send_message::Response {
    let finalised = args.finalised;
    let bot_id = args.bot_id;
    let bot_name = args.bot_name.clone();
    let user_message_id = args.user_message_id;
    let bot_caller = BotCaller {
        bot: args.bot_id,
        initiator: args.initiator.clone(),
    };

    let my_user_id = state.env.canister_id().into();
    let args: Args = args.into();
    let message_content: MessageContent = args.content.clone().into();

    if !state.data.is_bot_permitted(
        &bot_id,
        &bot_caller.initiator,
        BotPermissions::from_message_permission((&args.content).into()),
    ) {
        return c2c_bot_send_message::Response::Error(OCErrorCode::InitiatorNotAuthorized.into());
    }

    let result = match prepare(&args, true, state) {
        Ok(ok) => ok,
        Err(error) => return c2c_bot_send_message::Response::Error(error),
    };

    let now = result.now;

    let content = match MessageContentInternal::validate_new_message(args.content, true, UserType::BotV2, args.forwarding, now)
    {
        ValidateNewMessageContentResult::Success(content) => content,
        ValidateNewMessageContentResult::SuccessP2PSwap(_)
        | ValidateNewMessageContentResult::SuccessCrypto(_)
        | ValidateNewMessageContentResult::SuccessPrize(_) => unreachable!(),
        ValidateNewMessageContentResult::Error(error) => {
            return c2c_bot_send_message::Response::Error(OCErrorCode::InvalidMessageContent.with_json(&error));
        }
    };

    // Check if a message with the same id already exists
    if let Some(chat) = state.data.direct_chats.get_mut(&bot_id.into()) {
        if let Some((message, _)) =
            chat.events
                .message_internal(EventIndex::default(), args.thread_root_message_index, args.message_id.into())
        {
            // If the message id of a bot message matches an existing unfinalised bot message
            // then edit this message instead of pushing a new one
            if let Some(bot_message) = message.bot_context() {
                if bot_caller.bot == message.sender
                    && bot_caller.initiator.user() == bot_message.command.as_ref().map(|c| c.initiator)
                    && bot_caller.initiator.command() == bot_message.command.as_ref()
                    && !bot_message.finalised
                {
                    let edit_message_args = EditMessageArgs {
                        sender: bot_caller.bot,
                        min_visible_event_index: EventIndex::default(),
                        thread_root_message_index: args.thread_root_message_index,
                        message_id: args.message_id,
                        content,
                        block_level_markdown: Some(args.block_level_markdown),
                        finalise_bot_message: finalised,
                        now,
                    };

                    let Ok(EditMessageSuccess {
                        message_index, event, ..
                    }) = chat.events.edit_message::<UserEventPusher>(edit_message_args, None)
                    else {
                        // Shouldn't happen
                        return c2c_bot_send_message::Response::Error(OCErrorCode::InitiatorNotAuthorized.into());
                    };

                    if finalised && !chat.notifications_muted.value {
                        let message_type = message_content.content_type().to_string();
                        let message_text = message_content.notification_text(&[], &[]);
                        let image_url = message_content.notification_image_url();

                        let fcm_data = FcmData::for_direct_chat(bot_id)
                            .set_body_with_alt(&message_text, &message_type)
                            .set_optional_image(image_url.clone())
                            .set_sender_name(bot_name.clone());

                        let notification = UserNotificationPayload::DirectMessage(DirectMessageNotification {
                            sender: bot_id,
                            thread_root_message_index: args.thread_root_message_index,
                            message_index,
                            event_index: event.index,
                            sender_name: bot_name,
                            sender_display_name: None,
                            message_type,
                            message_text,
                            image_url,
                            sender_avatar_id: None,
                            crypto_transfer: message_content.notification_crypto_transfer_details(&[]),
                        });
                        state.push_notification(Some(bot_id), my_user_id, notification, fcm_data);
                    }

                    return c2c_bot_send_message::Response::Success(SuccessResult {
                        chat_id: bot_id.into(),
                        event_index: event.index,
                        message_index,
                        expires_at: event.expires_at,
                        timestamp: now,
                    });
                }
            }

            return c2c_bot_send_message::Response::Error(OCErrorCode::MessageAlreadyFinalized.into());
        }
    }

    // If the user_message_id is set, then the user is sending a direct message to the bot.
    // In which case rather than just posting the bot's message, we should first post the user's message.
    // This allows the user to have a more natural conversation with the bot rather than using a /command.
    let mut user_message = false;
    if let Some(command) = bot_caller.initiator.command() {
        if let (Some(text), Some(message_id)) = (
            command.args.first().and_then(|a| a.value.as_string().map(String::from)),
            user_message_id,
        ) {
            let chat = state
                .data
                .direct_chats
                .get_or_create(bot_id, UserType::BotV2, || state.env.rng().r#gen(), now);

            chat.push_message::<UserEventPusher>(
                PushMessageArgs {
                    thread_root_message_index: args.thread_root_message_index,
                    message_id,
                    sender: my_user_id,
                    content: MessageContentInternal::Text(TextContentInternal { text }),
                    mentioned: Vec::new(),
                    replies_to: None,
                    forwarded: false,
                    sender_is_bot: false,
                    block_level_markdown: args.block_level_markdown,
                    now,
                    sender_context: None,
                },
                None,
                None,
            );

            user_message = true;
        }
    }

    let event_wrapper = handle_message_impl(
        HandleMessageArgs {
            sender: bot_id,
            thread_root_message_id: None,
            message_id: Some(args.message_id),
            sender_message_index: None,
            sender_name: bot_name,
            sender_display_name: None,
            content,
            replies_to: None,
            forwarding: false,
            sender_user_type: UserType::BotV2,
            sender_avatar_id: None,
            push_message_sent_event: true,
            mentioned: Vec::new(),
            mute_notification: !finalised,
            block_level_markdown: args.block_level_markdown,
            now,
        },
        user_message.not().then_some(bot_caller),
        finalised,
        state,
    );

    c2c_bot_send_message::Response::Success(SuccessResult {
        chat_id: bot_id.into(),
        event_index: event_wrapper.index,
        message_index: event_wrapper.event.message_index,
        expires_at: event_wrapper.expires_at,
        timestamp: now,
    })
}

#[derive(Copy, Clone)]
enum RecipientType {
    _Self,
    Other(UserType),
}

impl RecipientType {
    fn is_self(&self) -> bool {
        matches!(self, RecipientType::_Self)
    }

    fn user_type(self) -> UserType {
        self.into()
    }
}

impl From<RecipientType> for UserType {
    fn from(value: RecipientType) -> Self {
        match value {
            RecipientType::_Self => UserType::User,
            RecipientType::Other(u) => u,
        }
    }
}

struct PrepareOk {
    my_user_id: UserId,
    now: TimestampMillis,
    local_user_index_canister_id: CanisterId,
    maybe_recipient_type: Option<RecipientType>,
}

fn prepare(args: &Args, is_v2_bot: bool, state: &RuntimeState) -> OCResult<PrepareOk> {
    state.data.verify_not_suspended()?;

    if state.data.blocked_users.contains(&args.recipient) {
        return Err(OCErrorCode::TargetUserBlocked.into());
    }

    if args.recipient == OPENCHAT_BOT_USER_ID {
        return Err(OCErrorCode::InvalidRequest.with_message("Messaging the OpenChat Bot is not currently supported"));
    }

    let my_user_id = state.env.canister_id().into();
    let maybe_recipient_type = if let Some(chat) = state.data.direct_chats.get(&args.recipient.into()) {
        if chat
            .events
            .message_already_finalised(args.thread_root_message_index, args.message_id, is_v2_bot)
        {
            return Err(OCErrorCode::MessageIdAlreadyExists.into());
        }
        Some(if args.recipient == my_user_id {
            RecipientType::_Self
        } else {
            RecipientType::Other(chat.user_type)
        })
    } else {
        None
    };

    Ok(PrepareOk {
        my_user_id,
        now: state.env.now(),
        local_user_index_canister_id: state.data.local_user_index_canister_id,
        maybe_recipient_type,
    })
}

#[expect(clippy::too_many_arguments)]
fn send_message_impl(
    my_user_id: UserId,
    recipient: UserId,
    thread_root_message_index: Option<MessageIndex>,
    message_id: MessageId,
    content: MessageContentInternal,
    replies_to: Option<ReplyContext>,
    forwarding: bool,
    block_level_markdown: bool,
    message_filter_failed: Option<u64>,
    recipient_type: RecipientType,
    completed_transfer: Option<CompletedCryptoTransaction>,
    state: &mut RuntimeState,
) -> Response {
    let now = state.env.now();
    let reply_context = replies_to.as_ref().map(ReplyContextInternal::from);

    let chat_private_replying_to = if let Some((chat, None)) = reply_context.as_ref().and_then(|r| r.chat_if_other) {
        Some(chat)
    } else {
        None
    };

    let push_message_args = PushMessageArgs {
        thread_root_message_index,
        message_id,
        sender: my_user_id,
        content: content.clone(),
        mentioned: Vec::new(),
        replies_to: reply_context,
        forwarded: forwarding,
        sender_is_bot: false,
        block_level_markdown,
        now,
        sender_context: None,
    };

    let chat = state
        .data
        .direct_chats
        .get_or_create(recipient, recipient_type.into(), || state.env.rng().r#gen(), now);

    let message_event = chat.push_message(
        push_message_args,
        None,
        Some(UserEventPusher {
            now,
            rng: state.env.rng(),
            queue: &mut state.data.local_user_index_event_sync_queue,
        }),
    );

    if !recipient_type.is_self() {
        let send_message_args = SendMessageArgs {
            thread_root_message_id: thread_root_message_index.map(|i| chat.main_message_index_to_id(i)),
            message_id,
            sender_message_index: message_event.event.message_index,
            content,
            replies_to: replies_to.and_then(|r| {
                if let Some((chat, thread_root_message_index)) = r.chat_if_other {
                    Some(C2CReplyContext::OtherChat(chat, thread_root_message_index, r.event_index))
                } else {
                    chat.events
                        .main_events_reader()
                        .message_internal(r.event_index.into())
                        .map(|m| m.message_id)
                        .map(C2CReplyContext::ThisChat)
                }
            }),
            forwarding,
            block_level_markdown,
            message_filter_failed,
        };

        let sender_name = state.data.username.value.clone();
        let sender_display_name = state.data.display_name.value.clone();

        if recipient_type.user_type().is_bot() {
            ic_cdk::futures::spawn(send_to_bot_canister(
                recipient,
                message_event.event.message_index,
                legacy_bot_api::handle_direct_message::Args::new(send_message_args, sender_name),
            ));
        } else {
            state.push_user_canister_event(
                recipient.into(),
                UserCanisterEvent::SendMessages(Box::new(SendMessagesArgs {
                    messages: vec![send_message_args],
                    sender_name,
                    sender_display_name,
                    sender_avatar_id: state.data.avatar.value.as_ref().map(|d| d.id),
                })),
            );
        }

        state.award_achievements_and_notify(message_event.event.achievements(true, false), now);
    }

    register_timer_jobs(
        recipient.into(),
        thread_root_message_index,
        message_id,
        &message_event,
        Vec::new(),
        now,
        &mut state.data,
    );

    if let Some(chat) = chat_private_replying_to {
        state
            .data
            .direct_chats
            .mark_private_reply(recipient, chat, message_event.event.message_index);
    }

    if let Some(transfer) = completed_transfer {
        TransferSuccessV2(TransferSuccessV2Result {
            chat_id: recipient.into(),
            event_index: message_event.index,
            message_index: message_event.event.message_index,
            timestamp: now,
            expires_at: message_event.expires_at,
            transfer,
        })
    } else {
        Success(SuccessResult {
            chat_id: recipient.into(),
            event_index: message_event.index,
            message_index: message_event.event.message_index,
            timestamp: now,
            expires_at: message_event.expires_at,
        })
    }
}

async fn send_to_bot_canister(
    recipient: UserId,
    message_index: MessageIndex,
    args: legacy_bot_api::handle_direct_message::Args,
) {
    match legacy_bot_c2c_client::handle_direct_message(recipient.into(), &args).await {
        Ok(legacy_bot_api::handle_direct_message::Response::Success(result)) => {
            mutate_state(|state| {
                if let Some(chat) = state.data.direct_chats.get_mut(&recipient.into()) {
                    let now = state.env.now();
                    for message in result.messages {
                        let push_message_args = PushMessageArgs {
                            sender: recipient,
                            thread_root_message_index: None,
                            message_id: message.message_id.unwrap_or_else(|| state.env.rng().r#gen()),
                            content: message.content.into(),
                            mentioned: Vec::new(),
                            replies_to: None,
                            forwarded: false,
                            sender_is_bot: false,
                            block_level_markdown: args.block_level_markdown,
                            now,
                            sender_context: None,
                        };
                        chat.push_message(
                            push_message_args,
                            None,
                            Some(UserEventPusher {
                                now,
                                rng: state.env.rng(),
                                queue: &mut state.data.local_user_index_event_sync_queue,
                            }),
                        );

                        // Mark that the bot has read the message we just sent
                        chat.mark_read_up_to(message_index, false, now);
                    }
                }
            });
        }
        Err(_error) => {
            // TODO push message saying that the message failed to send
        }
    }
}

pub(crate) fn register_timer_jobs(
    chat_id: ChatId,
    thread_root_message_index: Option<MessageIndex>,
    message_id: MessageId,
    message_event: &EventWrapper<Message>,
    file_references: Vec<BlobReference>,
    now: TimestampMillis,
    data: &mut Data,
) {
    if !file_references.is_empty() {
        if let Some(expiry) = message_event.expires_at {
            data.timer_jobs.enqueue_job(
                TimerJob::DeleteFileReferences(DeleteFileReferencesJob { files: file_references }),
                expiry,
                now,
            );
        }
    }

    if let Some(expiry) = message_event.expires_at {
        data.handle_event_expiry(expiry, now);
    }

    if let MessageContent::P2PSwap(c) = &message_event.event.content {
        data.timer_jobs.enqueue_job(
            TimerJob::MarkP2PSwapExpired(Box::new(MarkP2PSwapExpiredJob {
                chat_id,
                thread_root_message_index,
                message_id,
            })),
            c.expires_at,
            now,
        );
    }
}
