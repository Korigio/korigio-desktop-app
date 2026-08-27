//! Home dashboard DTOs (camelCase for IPC).

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DashboardRepairRow {
    pub id: i64,
    pub repair_number: String,
    pub status: String,
    pub customer_name: String,
    pub customer_phone: Option<String>,
    pub updated_at: String,
    pub ready_at: Option<String>,
    pub days_in_status: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TodayCounts {
    pub received: i64,
    pub collected: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HomeDashboard {
    pub status_counts: Vec<StatusCount>,
    pub ready_for_pickup: Vec<DashboardRepairRow>,
    pub stale_repairs: Vec<DashboardRepairRow>,
    pub today: TodayCounts,
    pub stale_after_days: u32,
}
