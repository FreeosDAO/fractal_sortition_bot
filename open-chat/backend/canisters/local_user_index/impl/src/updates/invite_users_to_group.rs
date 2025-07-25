use crate::guards::caller_is_openchat_user;
use crate::{RuntimeState, mutate_state, read_state};
use candid::Principal;
use canister_api_macros::update;
use canister_tracing_macros::trace;
use local_user_index_canister::invite_users_to_group::{Response::*, *};
use types::{ChatId, MessageContent, TextContent, UserId};

#[update(guard = "caller_is_openchat_user", candid = true, msgpack = true)]
#[trace]
async fn invite_users_to_group(args: Args) -> Response {
    let PrepareResult { invited_by, users } = read_state(|state| prepare(&args, state));

    let c2c_args = group_canister::c2c_invite_users::Args {
        caller: invited_by,
        users,
    };

    match group_canister_c2c_client::c2c_invite_users(args.group_id.into(), &c2c_args).await {
        Ok(response) => match response {
            group_canister::c2c_invite_users::Response::Success(s) => {
                mutate_state(|state| {
                    send_group_invitation(invited_by, args.group_id, s.group_name, s.invited_users, state);
                });
                Success
            }
            group_canister::c2c_invite_users::Response::Error(error) => Error(error),
        },
        Err(error) => InternalError(format!("Failed to call 'group::c2c_invite_users': {error:?}")),
    }
}

struct PrepareResult {
    invited_by: UserId,
    users: Vec<(UserId, Principal)>,
}

fn prepare(args: &Args, state: &RuntimeState) -> PrepareResult {
    let invited_by = state.calling_user_id();
    let users = args
        .user_ids
        .iter()
        .filter_map(|user_id| state.data.global_users.get(&(*user_id).into()))
        .map(|user| (user.user_id, user.principal))
        .collect();
    PrepareResult { invited_by, users }
}

pub(crate) fn send_group_invitation(
    invited_by: UserId,
    group_id: ChatId,
    group_name: String,
    invited_users: Vec<UserId>,
    state: &mut RuntimeState,
) {
    let now = state.env.now();
    let text = format!("You have been invited to the group [{group_name}](/group/{group_id}) by @UserId({invited_by}).");
    let message = MessageContent::Text(TextContent { text });

    for user_id in invited_users {
        state.push_oc_bot_message_to_user(user_id, message.clone(), now);
    }
}
