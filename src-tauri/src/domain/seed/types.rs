use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedSyntheticDataInput {
    pub customers: Option<u32>,
    pub devices: Option<u32>,
    pub repairs: Option<u32>,
    pub confirm: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SeedSyntheticDataResult {
    pub customers: u32,
    pub devices: u32,
    pub repairs: u32,
    pub elapsed_ms: u64,
}
