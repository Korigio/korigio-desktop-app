use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Repair {
    pub id: i64,
    pub repair_number: String,
    pub customer_id: i64,
    pub device_id: i64,
    /// Nullable in SQL for migrate safety on leftover rows; required on create.
    pub company_id: Option<i64>,
    pub status: String,
    pub received_at: String,
    pub reported_problem: Option<String>,
    pub accessories_received: Option<String>,
    pub device_condition: Option<String>,
    pub diagnosis_notes: Option<String>,
    pub work_performed: Option<String>,
    pub notes: Option<String>,
    pub expected_pickup_at: Option<String>,
    pub ready_at: Option<String>,
    pub collected_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairInput {
    pub customer_id: i64,
    pub device_id: i64,
    /// Required on create; ignored on update (company cannot change after create).
    pub company_id: i64,
    pub status: Option<String>,
    pub reported_problem: Option<String>,
    pub accessories_received: Option<String>,
    pub device_condition: Option<String>,
    pub diagnosis_notes: Option<String>,
    pub work_performed: Option<String>,
    pub notes: Option<String>,
    pub expected_pickup_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairListQuery {
    pub query: Option<String>,
    pub customer_id: Option<i64>,
    pub device_id: Option<i64>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairListResult {
    pub items: Vec<Repair>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}
