pub mod service;
pub mod types;

#[cfg(test)]
mod tests;

pub use service::get_repair_print_report;
pub use types::RepairPrintReport;
