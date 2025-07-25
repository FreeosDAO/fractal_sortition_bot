use oc_error_codes::OCError;
use serde::{Deserialize, Serialize};
use ts_export::ts_export;
use types::PublicGroupSummary;

#[ts_export(group, public_summary)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Args {
    pub invite_code: Option<u64>,
}

#[ts_export(group, public_summary)]
#[expect(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Success(SuccessResult),
    Error(OCError),
}

#[ts_export(group, public_summary)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SuccessResult {
    pub summary: PublicGroupSummary,
    pub is_invited: bool,
}
