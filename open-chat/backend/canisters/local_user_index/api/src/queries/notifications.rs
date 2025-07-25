use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use types::{IndexedEvent, NotificationEnvelope, NotificationSubscription, TimestampMillis, UserId};

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct Args {
    pub from_notification_index: u64,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub enum Response {
    Success(SuccessResult),
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct SuccessResult {
    pub notifications: Vec<IndexedEvent<NotificationEnvelope>>,
    pub subscriptions: HashMap<UserId, Vec<NotificationSubscription>>,
    pub bot_endpoints: HashMap<UserId, String>,
    pub timestamp: TimestampMillis,
}
