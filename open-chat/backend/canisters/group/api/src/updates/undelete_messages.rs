use oc_error_codes::OCError;
use serde::{Deserialize, Serialize};
use ts_export::ts_export;
use types::{Message, MessageId, MessageIndex};

#[ts_export(group, undelete_messages)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Args {
    pub thread_root_message_index: Option<MessageIndex>,
    pub message_ids: Vec<MessageId>,
}

#[ts_export(group, undelete_messages)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Success(SuccessResult),
    Error(OCError),
}

#[ts_export(group, undelete_messages)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SuccessResult {
    pub messages: Vec<Message>,
}
