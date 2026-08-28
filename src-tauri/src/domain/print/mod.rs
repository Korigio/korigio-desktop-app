pub mod service;
pub mod types;

#[cfg(test)]
mod tests;

pub use service::{
    get_repair_diagnosis_print_report, get_repair_print_report, get_repair_summary_print_report,
};
pub use types::{DiagnosisPrintReport, RepairPrintReport, SummaryPrintReport};
