use crate::{EventIndex, TimestampMillis};
use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use ts_export::ts_export;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct EventWrapper<T> {
    pub index: EventIndex,
    pub timestamp: TimestampMillis,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<TimestampMillis>,
    pub event: T,
}

#[derive(Clone, Debug)]
pub struct EventMetaData {
    pub index: EventIndex,
    pub timestamp: TimestampMillis,
    pub expires_at: Option<TimestampMillis>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventWrapperInternal<T> {
    #[serde(rename = "i", alias = "index")]
    pub index: EventIndex,
    #[serde(rename = "t", alias = "timestamp")]
    pub timestamp: TimestampMillis,
    #[serde(rename = "x", alias = "expires_at", default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<TimestampMillis>,
    #[serde(rename = "e", alias = "event")]
    pub event: T,
}

impl<T> EventWrapperInternal<T> {
    pub fn is_expired(&self, now: TimestampMillis) -> bool {
        self.expires_at.is_some_and(|expiry| expiry < now)
    }
}

impl<T> From<EventWrapperInternal<T>> for EventWrapper<T> {
    fn from(value: EventWrapperInternal<T>) -> Self {
        EventWrapper {
            index: value.index,
            timestamp: value.timestamp,
            expires_at: value.expires_at,
            event: value.event,
        }
    }
}

macro_rules! event_wrapper {
    ($name:ident, $event_type:ty) => {
        #[ts_export]
        #[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
        pub struct $name {
            pub index: EventIndex,
            pub timestamp: TimestampMillis,
            pub expires_at: Option<TimestampMillis>,
            pub event: $event_type,
        }
    };
}

event_wrapper!(EventWrapperChatEvent, crate::ChatEvent);
event_wrapper!(EventWrapperGroupFrozen, crate::GroupFrozen);
event_wrapper!(EventWrapperGroupUnfrozen, crate::GroupUnfrozen);
event_wrapper!(EventWrapperMessage, crate::Message);
event_wrapper!(EventWrapperCommunityEvent, crate::CommunityEvent);
