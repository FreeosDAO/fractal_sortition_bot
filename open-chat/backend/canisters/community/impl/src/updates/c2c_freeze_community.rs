use crate::activity_notifications::handle_activity_notification;
use crate::guards::caller_is_group_index_or_local_user_index;
use crate::model::events::CommunityEventInternal;
use crate::{RuntimeState, execute_update};
use canister_api_macros::update;
use canister_tracing_macros::trace;
use community_canister::c2c_freeze_community::{Response::*, *};
use types::{EventWrapper, FrozenGroupInfo, GroupFrozen, Timestamped};

#[update(guard = "caller_is_group_index_or_local_user_index", msgpack = true)]
#[trace]
fn c2c_freeze_community(args: Args) -> Response {
    execute_update(|state| c2c_freeze_community_impl(args, state))
}

fn c2c_freeze_community_impl(args: Args, state: &mut RuntimeState) -> Response {
    if state.data.frozen.is_none() {
        let now = state.env.now();

        state.data.frozen = Timestamped::new(
            Some(FrozenGroupInfo {
                timestamp: now,
                frozen_by: args.caller,
                reason: args.reason.clone(),
            }),
            now,
        );

        let event_index = state.push_community_event(CommunityEventInternal::Frozen(Box::new(GroupFrozen {
            frozen_by: args.caller,
            reason: args.reason.clone(),
        })));

        let event = EventWrapper {
            index: event_index,
            timestamp: now,
            expires_at: None,
            event: GroupFrozen {
                frozen_by: args.caller,
                reason: args.reason,
            },
        };

        handle_activity_notification(state);

        if args.return_members {
            SuccessWithMembers(event, state.data.members.iter_member_ids().collect())
        } else {
            Success(event)
        }
    } else {
        CommunityAlreadyFrozen
    }
}
