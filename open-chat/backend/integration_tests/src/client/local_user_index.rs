use crate::{generate_msgpack_query_call, generate_msgpack_update_call, generate_query_call, generate_update_call};
use local_user_index_canister::*;

// Queries
generate_msgpack_query_call!(access_token_v2);
generate_query_call!(bot_chat_events);
generate_query_call!(bot_community_events);
generate_msgpack_query_call!(chat_events);
generate_msgpack_query_call!(group_and_community_summary_updates_v2);
generate_query_call!(latest_notification_index);
generate_query_call!(notifications);

// Updates
generate_update_call!(bot_create_channel);
generate_update_call!(bot_delete_channel);
generate_update_call!(bot_send_message);
generate_update_call!(bot_subscribe_to_events);
generate_msgpack_update_call!(install_bot);
generate_msgpack_update_call!(invite_users_to_channel);
generate_msgpack_update_call!(invite_users_to_community);
generate_msgpack_update_call!(invite_users_to_group);
generate_msgpack_update_call!(join_channel);
generate_msgpack_update_call!(join_community);
generate_msgpack_update_call!(join_group);
generate_msgpack_update_call!(register_user);
generate_msgpack_update_call!(uninstall_bot);

pub mod happy_path {
    use crate::User;
    use crate::utils::{principal_to_username, tick_many};
    use candid::Principal;
    use local_user_index_canister::{install_bot, uninstall_bot};
    use pocket_ic::PocketIc;
    use types::{
        BotInstallationLocation, BotPermissions, CanisterId, ChannelId, ChatId, CommunityCanisterCommunitySummary, CommunityId,
        Empty, UserId,
    };

    pub fn register_user(env: &mut PocketIc, principal: Principal, canister_id: CanisterId, public_key: Vec<u8>) -> User {
        register_user_with_referrer(env, principal, canister_id, public_key, None)
    }

    pub fn register_user_with_referrer(
        env: &mut PocketIc,
        principal: Principal,
        canister_id: CanisterId,
        public_key: Vec<u8>,
        referral_code: Option<String>,
    ) -> User {
        let response = super::register_user(
            env,
            principal,
            canister_id,
            &local_user_index_canister::register_user::Args {
                username: principal_to_username(principal),
                referral_code,
                public_key: public_key.clone(),
            },
        );

        tick_many(env, 3);

        match response {
            local_user_index_canister::register_user::Response::Success(res) => User {
                principal,
                user_id: res.user_id,
                public_key,
                local_user_index: canister_id,
            },
            response => panic!("'register_user' error: {response:?}"),
        }
    }

    pub fn invite_users_to_group(
        env: &mut PocketIc,
        user: &User,
        local_user_index_canister_id: CanisterId,
        group_id: ChatId,
        user_ids: Vec<UserId>,
    ) {
        let response = super::invite_users_to_group(
            env,
            user.principal,
            local_user_index_canister_id,
            &local_user_index_canister::invite_users_to_group::Args { group_id, user_ids },
        );

        match response {
            local_user_index_canister::invite_users_to_group::Response::Success => {}
            response => panic!("'invite_users_to_group' error: {response:?}"),
        }
    }

    pub fn join_group(env: &mut PocketIc, sender: Principal, local_user_index_canister_id: CanisterId, chat_id: ChatId) {
        let response = super::join_group(
            env,
            sender,
            local_user_index_canister_id,
            &local_user_index_canister::join_group::Args {
                chat_id,
                invite_code: None,
                verified_credential_args: None,
            },
        );

        match response {
            local_user_index_canister::join_group::Response::Success(_) => {}
            response => panic!("'join_group' error: {response:?}"),
        }
    }

    pub fn add_users_to_group(
        env: &mut PocketIc,
        user: &User,
        local_user_index_canister_id: CanisterId,
        group_id: ChatId,
        users: Vec<(UserId, Principal)>,
    ) {
        invite_users_to_group(
            env,
            user,
            local_user_index_canister_id,
            group_id,
            users.iter().map(|(user_id, _)| *user_id).collect(),
        );

        for (_, principal) in users {
            join_group(env, principal, local_user_index_canister_id, group_id);
        }

        env.tick();
    }

    pub fn invite_users_to_community(
        env: &mut PocketIc,
        user: &User,
        local_user_index_canister_id: CanisterId,
        community_id: CommunityId,
        user_ids: Vec<UserId>,
    ) {
        let response = super::invite_users_to_community(
            env,
            user.principal,
            local_user_index_canister_id,
            &local_user_index_canister::invite_users_to_community::Args { community_id, user_ids },
        );

        match response {
            local_user_index_canister::invite_users_to_community::Response::Success => {}
            response => panic!("'invite_users_to_community' error: {response:?}"),
        }
    }

    pub fn invite_users_to_channel(
        env: &mut PocketIc,
        user: &User,
        local_user_index_canister_id: CanisterId,
        community_id: CommunityId,
        channel_id: ChannelId,
        user_ids: Vec<UserId>,
    ) {
        let response = super::invite_users_to_channel(
            env,
            user.principal,
            local_user_index_canister_id,
            &local_user_index_canister::invite_users_to_channel::Args {
                community_id,
                channel_id,
                user_ids,
            },
        );

        match response {
            local_user_index_canister::invite_users_to_channel::Response::Success => {}
            response => panic!("'invite_users_to_channel' error: {response:?}"),
        }
    }

    pub fn join_community(
        env: &mut PocketIc,
        sender: Principal,
        local_user_index_canister_id: CanisterId,
        community_id: CommunityId,
        referred_by: Option<UserId>,
    ) -> CommunityCanisterCommunitySummary {
        let response = super::join_community(
            env,
            sender,
            local_user_index_canister_id,
            &local_user_index_canister::join_community::Args {
                community_id,
                invite_code: None,
                referred_by,
                verified_credential_args: None,
            },
        );

        match response {
            local_user_index_canister::join_community::Response::Success(result) => *result,
            response => panic!("'join_community' error: {response:?}"),
        }
    }

    pub fn join_channel(
        env: &mut PocketIc,
        sender: Principal,
        local_user_index_canister_id: CanisterId,
        community_id: CommunityId,
        channel_id: ChannelId,
    ) {
        let response = super::join_channel(
            env,
            sender,
            local_user_index_canister_id,
            &local_user_index_canister::join_channel::Args {
                community_id,
                channel_id,
                invite_code: None,
                referred_by: None,
                verified_credential_args: None,
            },
        );

        match response {
            local_user_index_canister::join_channel::Response::Success(_)
            | local_user_index_canister::join_channel::Response::SuccessJoinedCommunity(_)
            | local_user_index_canister::join_channel::Response::AlreadyInChannel(_) => {}
            response => panic!("'join_channel' error: {response:?}"),
        }
    }

    pub fn add_users_to_community(
        env: &mut PocketIc,
        user: &User,
        local_user_index_canister_id: CanisterId,
        community_id: CommunityId,
        users: Vec<(UserId, Principal)>,
    ) {
        invite_users_to_community(
            env,
            user,
            local_user_index_canister_id,
            community_id,
            users.iter().map(|(user_id, _)| *user_id).collect(),
        );

        for (_, principal) in users {
            join_community(env, principal, local_user_index_canister_id, community_id, None);
        }

        env.tick();
    }

    pub fn access_token(
        env: &PocketIc,
        sender: &User,
        local_user_index_canister_id: CanisterId,
        args: &local_user_index_canister::access_token_v2::Args,
    ) -> String {
        let response = super::access_token_v2(env, sender.principal, local_user_index_canister_id, args);

        match response {
            local_user_index_canister::access_token_v2::Response::Success(token) => token,
            response => panic!("'access_token' error: {response:?}"),
        }
    }

    pub fn install_bot(
        env: &mut PocketIc,
        sender: Principal,
        local_user_index: CanisterId,
        location: BotInstallationLocation,
        bot_id: UserId,
        granted_permissions: BotPermissions,
        granted_autonomous_permissions: Option<BotPermissions>,
    ) {
        let response = super::install_bot(
            env,
            sender,
            local_user_index,
            &install_bot::Args {
                bot_id,
                granted_permissions,
                location,
                granted_autonomous_permissions,
            },
        );

        match response {
            install_bot::Response::Success => {}
            response => panic!("'install_bot' error: {response:?}"),
        }
    }

    pub fn uninstall_bot(
        env: &mut PocketIc,
        sender: Principal,
        local_user_index: CanisterId,
        location: BotInstallationLocation,
        bot_id: UserId,
    ) {
        let response = super::uninstall_bot(env, sender, local_user_index, &uninstall_bot::Args { bot_id, location });

        match response {
            uninstall_bot::Response::Success => {}
            response => panic!("'update_bot' error: {response:?}"),
        }
    }

    pub fn notifications(
        env: &PocketIc,
        sender: Principal,
        local_user_index: CanisterId,
        from_index: u64,
    ) -> local_user_index_canister::notifications::SuccessResult {
        let response = super::notifications(
            env,
            sender,
            local_user_index,
            &local_user_index_canister::notifications::Args {
                from_notification_index: from_index,
            },
        );

        let local_user_index_canister::notifications::Response::Success(result) = response;
        result
    }

    pub fn latest_notification_index(env: &PocketIc, sender: Principal, local_user_index: CanisterId) -> u64 {
        let response = super::latest_notification_index(env, sender, local_user_index, &Empty {});
        let local_user_index_canister::latest_notification_index::Response::Success(index) = response;
        index
    }
}
