use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Repair {
    pub id: String,
    pub repair_number: String,
    pub customer_id: String,
    pub device_id: String,
    /// Nullable in SQL for migrate safety on leftover rows; required on create.
    pub company_id: Option<String>,
    pub assigned_to_staff_id: Option<String>,
    pub updated_by_staff_id: Option<String>,
    pub status: String,
    pub received_at: String,
    pub reported_problem: Option<String>,
    pub accessories_received: Option<String>,
    pub device_condition: Option<String>,
    pub diagnosis_notes: Option<String>,
    pub work_performed: Option<String>,
    pub notes: Option<String>,
    pub expected_pickup_at: Option<String>,
    /// Pre-discount list price; null for legacy rows without list/discount.
    #[serde(default)]
    pub estimate_list_cents: Option<i64>,
    /// Discount in basis points; null for legacy rows without list/discount.
    #[serde(default)]
    pub estimate_discount_bps: Option<i64>,
    /// Pre-tax net estimate in integer cents; null until set at intake or diagnosis.
    pub estimate_base_cents: Option<i64>,
    /// Tax rate snapshotted at estimate time, in basis points (e.g. 19% → 1900).
    pub estimate_tax_rate_bps: Option<i64>,
    pub estimate_tax_cents: Option<i64>,
    pub estimate_gross_cents: Option<i64>,
    pub ready_at: Option<String>,
    pub collected_at: Option<String>,
    /// Whole years of warranty; null until summary handover. Range `0..=10` when set.
    #[serde(default)]
    pub warranty_years: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RepairDocumentType {
    EntranceSigned,
    DiagnosisSigned,
    SummarySigned,
}

impl RepairDocumentType {
    pub fn as_slug(self) -> &'static str {
        match self {
            Self::EntranceSigned => "entrance_signed",
            Self::DiagnosisSigned => "diagnosis_signed",
            Self::SummarySigned => "summary_signed",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "entrance_signed" => Some(Self::EntranceSigned),
            "diagnosis_signed" => Some(Self::DiagnosisSigned),
            "summary_signed" => Some(Self::SummarySigned),
            _ => None,
        }
    }

    pub fn to_camel(self) -> &'static str {
        match self {
            Self::EntranceSigned => "entranceSigned",
            Self::DiagnosisSigned => "diagnosisSigned",
            Self::SummarySigned => "summarySigned",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepairDocument {
    pub repair_id: String,
    pub document_type: String,
    pub original_filename: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompleteDiagnosisMode {
    Draft,
    Finalize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteRepairDiagnosisInput {
    pub repair_id: String,
    pub mode: CompleteDiagnosisMode,
    pub diagnosis_notes: Option<String>,
    pub expected_pickup_at: Option<String>,
    /// List price in cents (same as create_repair). Null only when never set; clearing rejected.
    pub estimate_base_cents: Option<i64>,
    /// Discount in basis points (`0..=10000`). `None` = 0. Error if set without list.
    #[serde(default)]
    pub estimate_discount_bps: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteRepairDiagnosisResult {
    pub repair: Repair,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairInput {
    pub customer_id: String,
    pub device_id: String,
    /// Required on create; ignored on update (company cannot change after create).
    pub company_id: String,
    pub status: Option<String>,
    pub reported_problem: Option<String>,
    pub accessories_received: Option<String>,
    pub device_condition: Option<String>,
    pub diagnosis_notes: Option<String>,
    pub work_performed: Option<String>,
    pub notes: Option<String>,
    pub expected_pickup_at: Option<String>,
    /// Create only: pre-discount list price in cents. `None` = no intake estimate.
    /// Ignored on `update_repair`.
    #[serde(default)]
    pub estimate_base_cents: Option<i64>,
    /// Create only: discount in basis points (`0..=10000`). `None` = 0.
    /// Error if set without `estimate_base_cents`. Ignored on `update_repair`.
    #[serde(default)]
    pub estimate_discount_bps: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairListQuery {
    pub query: Option<String>,
    pub customer_id: Option<String>,
    pub device_id: Option<String>,
    pub company_id: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Repair row enriched with customer display data for list views.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepairListItem {
    #[serde(flatten)]
    pub repair: Repair,
    pub customer_name: String,
    pub device_name: String,
}

impl std::ops::Deref for RepairListItem {
    type Target = Repair;

    fn deref(&self) -> &Self::Target {
        &self.repair
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairListResult {
    pub items: Vec<RepairListItem>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}
