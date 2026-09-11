pub mod constants;
pub mod documents;
pub mod money;
pub mod repository;
pub mod service;
pub mod types;
pub mod validation;

#[cfg(test)]
mod tests;

pub use documents::{
    delete_repair_document, list_repair_documents, open_repair_document,
    resolve_repair_document_absolute, upload_repair_document,
};
pub use service::{
    assign_repair, complete_repair_diagnosis, complete_repair_pickup, complete_repair_protocol,
    confirm_customer_approval, confirm_repair_intake, confirm_repair_parts_received,
    confirm_repair_summary, create_repair, get_repair, list_repairs,
    record_repair_summary_handover, take_over_repair, update_repair,
};
pub use types::{
    CompleteRepairDiagnosisInput, CompleteRepairDiagnosisResult, Repair, RepairDocument,
    RepairDocumentType, RepairInput, RepairListItem, RepairListQuery, RepairListResult,
};
