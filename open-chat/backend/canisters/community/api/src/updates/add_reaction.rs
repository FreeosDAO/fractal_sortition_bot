use serde::{Deserialize, Serialize};
use ts_export::ts_export;
use types::{ChannelId, MessageId, MessageIndex, Reaction, UnitResult};

#[ts_export(community, add_reaction)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Args {
    pub channel_id: ChannelId,
    pub thread_root_message_index: Option<MessageIndex>,
    pub message_id: MessageId,
    pub reaction: Reaction,
    pub username: String,
    pub display_name: Option<String>,
    pub new_achievement: bool,
}

pub type Response = UnitResult;
