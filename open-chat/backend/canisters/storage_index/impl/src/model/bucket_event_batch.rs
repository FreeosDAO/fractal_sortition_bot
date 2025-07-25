use candid::Principal;
use serde::{Deserialize, Serialize};
use timer_job_queues::{TimerJobItem, grouped_timer_job_batch};
use types::{AccessorId, CanisterId, FileId, Milliseconds};
use utils::canister::delay_if_should_retry_failed_c2c_call;

grouped_timer_job_batch!(BucketEventBatch, CanisterId, EventToSync, 1000);

#[derive(Serialize, Deserialize, Clone)]
pub enum EventToSync {
    UserAdded(Principal),
    UserRemoved(Principal),
    AccessorRemoved(AccessorId),
    UserIdUpdated(Principal, Principal),
    FileToRemove(FileId),
}

impl TimerJobItem for BucketEventBatch {
    async fn process(&self) -> Result<(), Option<Milliseconds>> {
        let mut args = storage_bucket_canister::c2c_sync_index::Args::default();
        for event in &self.items {
            match event {
                EventToSync::UserAdded(a) => args.users_added.push(*a),
                EventToSync::UserRemoved(r) => args.users_removed.push(*r),
                EventToSync::AccessorRemoved(r) => args.accessors_removed.push(*r),
                EventToSync::UserIdUpdated(old, new) => args.user_ids_updated.push((*old, *new)),
                EventToSync::FileToRemove(file_id) => args.files_to_remove.push(*file_id),
            }
        }

        let response = storage_bucket_canister_c2c_client::c2c_sync_index(self.key, &args).await;

        match response {
            Ok(_) => Ok(()),
            Err(error) => {
                let delay_if_should_retry = delay_if_should_retry_failed_c2c_call(error.reject_code(), error.message());
                Err(delay_if_should_retry)
            }
        }
    }
}
