use crate::lifecycle::{init_env, init_state};
use crate::memory::get_stable_memory_map_memory;
use crate::{Data, mutate_state, openchat_bot};
use canister_tracing_macros::trace;
use ic_cdk::init;
use tracing::info;
use user_canister::init::Args;
use utils::env::Environment;

#[init]
#[trace]
fn init(args: Args) {
    canister_logger::init(args.test_mode);
    stable_memory_map::init(get_stable_memory_map_memory());

    let env = init_env([0; 32]);
    let now = env.now();

    let data = Data::new(
        args.owner,
        args.user_index_canister_id,
        args.local_user_index_canister_id,
        args.group_index_canister_id,
        args.identity_canister_id,
        args.escrow_canister_id,
        args.video_call_operators,
        args.username,
        args.test_mode,
        args.referred_by,
        now,
    );

    init_state(env, data, args.wasm_version);

    mutate_state(|state| {
        for message in args.openchat_bot_messages {
            openchat_bot::send_message(message.into(), Vec::new(), true, state);
        }
    });

    info!(version = %args.wasm_version, "Initialization complete");
}
