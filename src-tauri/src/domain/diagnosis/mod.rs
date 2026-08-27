pub mod constants;
pub mod repository;
pub mod service;
pub mod types;
pub mod validation;

#[cfg(test)]
mod tests;

pub use service::{
    create_diagnosis_template, delete_diagnosis_template, get_diagnosis_template,
    get_repair_diagnosis, list_diagnosis_templates, update_diagnosis_template,
    upsert_repair_diagnosis,
};
pub use types::{
    DiagnosisTemplate, DiagnosisTemplateInput, DiagnosisTemplateListQuery,
    DiagnosisTemplateListResult, RepairDiagnosis, RepairDiagnosisInput,
};
