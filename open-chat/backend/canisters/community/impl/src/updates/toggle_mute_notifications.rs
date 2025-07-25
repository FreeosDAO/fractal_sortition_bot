use crate::{RuntimeState, execute_update, model::channels::MuteChannelResult};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use community_canister::toggle_mute_notifications::*;
use oc_error_codes::OCErrorCode;
use types::OCResult;

#[update(msgpack = true)]
#[trace]
fn toggle_mute_notifications(args: Args) -> Response {
    execute_update(|state| toggle_mute_notifications_impl(args, state)).into()
}

fn toggle_mute_notifications_impl(args: Args, state: &mut RuntimeState) -> OCResult {
    state.data.verify_not_frozen()?;

    let user_id = state.get_caller_user_id()?;
    let now = state.env.now();

    let updated = if let Some(channel_id) = args.channel_id {
        let channel = state.data.channels.get_mut_or_err(&channel_id)?;
        match channel.mute_notifications(args.mute, user_id, now) {
            MuteChannelResult::Success => true,
            MuteChannelResult::Unchanged => false,
            MuteChannelResult::UserNotFound => return Err(OCErrorCode::InitiatorNotInChat.into()),
        }
    } else {
        // Mute (or unmute) all channels
        let mut updated = false;
        for channel in state.data.channels.iter_mut() {
            let result = channel.mute_notifications(args.mute, user_id, now);
            if matches!(result, MuteChannelResult::Success) {
                updated = true;
            }
        }
        updated
    };

    if updated {
        state.mark_activity_for_user(user_id);
    }
    Ok(())
}
