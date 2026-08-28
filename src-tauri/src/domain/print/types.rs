use serde::Serialize;

/// Assembled print payload for a repair ticket (no images).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RepairPrintReport {
    pub repair: PrintRepairCore,
    pub customer: PrintCustomer,
    pub device: PrintDevice,
    pub company: Option<PrintCompany>,
    pub company_logo_absolute_path: Option<String>,
    pub diagnosis: Option<PrintDiagnosis>,
}

/// Diagnosis paper for customer signature (notes + estimate; no checklist JSON).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosisPrintReport {
    pub repair: DiagnosisPrintRepair,
    pub customer: PrintCustomer,
    pub device: PrintDevice,
    pub company: Option<PrintCompany>,
    pub company_logo_absolute_path: Option<String>,
    /// Display currency from shop settings.
    pub currency: String,
}

/// Repair summary for pickup / completion paperwork.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SummaryPrintReport {
    pub repair: SummaryPrintRepair,
    pub customer: PrintCustomer,
    pub device: PrintDevice,
    pub company: Option<PrintCompany>,
    pub company_logo_absolute_path: Option<String>,
    /// Display currency from shop settings.
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrintRepairCore {
    pub id: i64,
    pub repair_number: String,
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
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosisPrintRepair {
    pub id: i64,
    pub repair_number: String,
    pub status: String,
    pub diagnosis_notes: Option<String>,
    pub expected_pickup_at: Option<String>,
    pub estimate_base_cents: Option<i64>,
    pub estimate_tax_rate_bps: Option<i64>,
    pub estimate_tax_cents: Option<i64>,
    pub estimate_gross_cents: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SummaryPrintRepair {
    pub id: i64,
    pub repair_number: String,
    pub status: String,
    pub received_at: String,
    pub reported_problem: Option<String>,
    pub accessories_received: Option<String>,
    pub device_condition: Option<String>,
    pub work_performed: Option<String>,
    pub expected_pickup_at: Option<String>,
    pub ready_at: Option<String>,
    pub collected_at: Option<String>,
    pub estimate_base_cents: Option<i64>,
    pub estimate_tax_rate_bps: Option<i64>,
    pub estimate_tax_cents: Option<i64>,
    pub estimate_gross_cents: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrintCustomer {
    pub id: i64,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrintDevice {
    pub id: i64,
    pub device_type: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrintCompany {
    pub id: i64,
    pub legal_name: String,
    pub trade_name: Option<String>,
    pub tax_id: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrintDiagnosis {
    pub items: Vec<PrintDiagnosisItem>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrintDiagnosisItem {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub value: PrintDiagnosisValue,
}

/// Print checklist value: boolean (checkbox) or string (text).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum PrintDiagnosisValue {
    Bool(bool),
    Text(String),
}
