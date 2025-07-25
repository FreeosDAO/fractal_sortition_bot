use serde::{Deserialize, Serialize};
use ts_export::ts_export;
use types::{BotPermissions, UnitResult, UserId};

#[ts_export(community, update_bot)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Args {
    pub bot_id: UserId,
    pub granted_permissions: BotPermissions,
    pub granted_autonomous_permissions: Option<BotPermissions>,
}

pub type Response = UnitResult;
