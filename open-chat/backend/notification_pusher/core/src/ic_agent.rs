use ic_agent::identity::{BasicIdentity, Secp256k1Identity};
use ic_agent::{Agent, Identity};
use local_user_index_canister::{latest_notification_index, notifications, remove_notifications};
use notifications_index_canister::remove_subscriptions;
use std::collections::HashMap;
use tracing::trace;
use types::{CanisterId, Empty, Error, UserId};

#[derive(Clone)]
pub struct IcAgent {
    agent: Agent,
}

impl IcAgent {
    pub async fn build(ic_url: &str, ic_identity_pem: &str, fetch_root_key: bool) -> Result<IcAgent, Error> {
        let timeout = std::time::Duration::from_secs(60 * 5);

        let agent = Agent::builder()
            .with_url(ic_url.to_string())
            .with_boxed_identity(Self::get_identity(ic_identity_pem))
            .with_ingress_expiry(timeout)
            .build()?;

        if fetch_root_key {
            agent.fetch_root_key().await?;
        }

        Ok(IcAgent { agent })
    }

    pub async fn notification_canisters(&self, notifications_index_canister_id: CanisterId) -> Result<Vec<CanisterId>, Error> {
        notifications_index_canister_client::notification_canisters(&self.agent, &notifications_index_canister_id, &Empty {})
            .await
            .inspect(|canister_ids| trace!(?canister_ids, "notification_canisters response"))
    }

    pub async fn notifications(
        &self,
        notifications_canister_id: &CanisterId,
        from_notification_index: u64,
    ) -> Result<notifications::SuccessResult, Error> {
        let args = notifications::Args { from_notification_index };

        trace!(?args, "notifications::args");

        let notifications::Response::Success(result) =
            local_user_index_canister_client::notifications(&self.agent, notifications_canister_id, &args).await?;

        trace!(?result, "notifications::result");

        Ok(result)
    }

    pub async fn latest_notifications_index(&self, notifications_canister_id: &CanisterId) -> Result<u64, Error> {
        let args = latest_notification_index::Args {};

        let latest_notification_index::Response::Success(index) =
            local_user_index_canister_client::latest_notification_index(&self.agent, notifications_canister_id, &args).await?;

        Ok(index)
    }

    pub async fn remove_notifications(
        &self,
        notifications_canister_id: &CanisterId,
        up_to_notification_index: u64,
    ) -> Result<(), Error> {
        let args = remove_notifications::Args {
            up_to_notification_index,
        };

        trace!(?args, "remove_notifications::args");

        local_user_index_canister_client::remove_notifications(&self.agent, notifications_canister_id, &args).await?;

        Ok(())
    }

    pub async fn remove_subscriptions(
        &self,
        index_canister_id: &CanisterId,
        subscriptions_by_user: HashMap<UserId, Vec<String>>,
    ) -> Result<(), Error> {
        if subscriptions_by_user.is_empty() {
            return Ok(());
        }

        let subscriptions_by_user = subscriptions_by_user
            .into_iter()
            .map(|(user_id, p256dh_keys)| remove_subscriptions::UserSubscriptions { user_id, p256dh_keys })
            .collect();

        let args = remove_subscriptions::Args { subscriptions_by_user };

        trace!(?args, "remove_subscriptions::args");

        notifications_index_canister_client::remove_subscriptions(&self.agent, index_canister_id, &args).await?;

        Ok(())
    }

    /// Returns an identity derived from the private key.
    fn get_identity(pem: &str) -> Box<dyn Identity + Sync + Send> {
        if let Ok(identity) = BasicIdentity::from_pem(pem.as_bytes()) {
            Box::new(identity)
        } else if let Ok(identity) = Secp256k1Identity::from_pem(pem.as_bytes()) {
            Box::new(identity)
        } else {
            panic!("Failed to create identity from pem");
        }
    }
}
