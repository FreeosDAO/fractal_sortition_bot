use crate::{RuntimeState, read_state};
use ic_cdk::inspect_message;

#[inspect_message]
fn inspect_message() {
    read_state(accept_if_valid);
}

fn accept_if_valid(state: &RuntimeState) {
    let method_name = ic_cdk::api::msg_method_name().trim_end_matches("_msgpack").to_string();

    let is_valid = match method_name.as_str() {
        "claim_daily_chit"
        | "create_canister"
        | "delete_user"
        | "mark_as_online"
        | "mark_suspected_bot"
        | "pay_for_diamond_membership"
        | "register_bot"
        | "set_display_name"
        | "set_moderation_flags"
        | "set_username"
        | "submit_proof_of_unique_personhood"
        | "update_bot"
        | "update_diamond_membership_subscription" => {
            let caller = state.env.caller();

            state.data.users.get_by_principal(&caller).is_some()
        }
        "suspend_user" | "unsuspend_user" => state.is_caller_platform_moderator(),
        "set_user_upgrade_concurrency" | "set_diamond_membership_fees" => state.is_caller_platform_operator(),
        "upload_wasm_chunk" => state.can_caller_upload_wasm_chunks(),
        "add_platform_moderator"
        | "add_platform_operator"
        | "remove_platform_moderator"
        | "remove_platform_operator"
        | "assign_platform_moderators_group"
        | "set_max_concurrent_user_canister_upgrades"
        | "add_local_user_index_canister"
        | "upgrade_user_canister_wasm"
        | "upgrade_local_user_index_canister_wasm"
        | "mark_local_user_index_full"
        | "register_external_achievement"
        | "publish_bot"
        | "suspected_bots" => state.is_caller_governance_principal(),
        "award_external_achievement" | "modclub_callback" => true,
        "remove_bot" => state.is_caller_governance_principal() || state.is_caller_openchat_user(),
        _ => false,
    };

    if is_valid {
        ic_cdk::api::accept_message();
    }
}
