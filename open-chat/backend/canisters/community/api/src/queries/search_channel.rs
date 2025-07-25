use oc_error_codes::OCError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use ts_export::ts_export;
use types::{ChannelId, MessageMatch, UserId};

#[ts_export(community, search_channel)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Args {
    pub channel_id: ChannelId,
    pub search_term: String,
    pub max_results: u8,
    pub users: Option<HashSet<UserId>>,
}

#[ts_export(community, search_channel)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Success(SuccessResult),
    Error(OCError),
}

#[ts_export(community, search_channel)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SuccessResult {
    pub matches: Vec<MessageMatch>,
}
