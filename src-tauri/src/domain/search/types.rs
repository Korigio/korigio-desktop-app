use serde::{Deserialize, Serialize};

use crate::domain::customers::Customer;
use crate::domain::devices::Device;
use crate::domain::repairs::Repair;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSearchQuery {
    pub query: String,
    pub limit_per_type: Option<u32>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSearchResult {
    pub customers: Vec<Customer>,
    pub devices: Vec<Device>,
    pub repairs: Vec<Repair>,
}
