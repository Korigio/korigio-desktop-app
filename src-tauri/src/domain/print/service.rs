use rusqlite::Connection;
use serde_json::Value;

use crate::domain::customers;
use crate::domain::devices;
use crate::domain::diagnosis;
use crate::domain::print::types::{
    PrintCustomer, PrintDevice, PrintDiagnosis, PrintDiagnosisItem, PrintDiagnosisValue,
    PrintRepairCore, RepairPrintReport,
};
use crate::domain::repairs;
use crate::error::AppError;

/// Assemble a print report for `repair_id`. Returns [`AppError::NotFound`] if the repair is missing.
pub fn get_repair_print_report(
    conn: &Connection,
    repair_id: i64,
) -> Result<RepairPrintReport, AppError> {
    let repair = repairs::get_repair(conn, repair_id)?;
    let customer = customers::get_customer(conn, repair.customer_id)?;
    let device = devices::get_device(conn, repair.device_id)?;
    let diagnosis = diagnosis::get_repair_diagnosis(conn, repair_id)?.map(|d| PrintDiagnosis {
        items: d
            .result
            .items
            .into_iter()
            .map(|item| PrintDiagnosisItem {
                id: item.id,
                label: item.label,
                kind: item.kind,
                value: map_diagnosis_value(item.value),
            })
            .collect(),
    });

    Ok(RepairPrintReport {
        repair: PrintRepairCore {
            id: repair.id,
            repair_number: repair.repair_number,
            status: repair.status,
            received_at: repair.received_at,
            reported_problem: repair.reported_problem,
            accessories_received: repair.accessories_received,
            device_condition: repair.device_condition,
            diagnosis_notes: repair.diagnosis_notes,
            work_performed: repair.work_performed,
            notes: repair.notes,
            expected_pickup_at: repair.expected_pickup_at,
            ready_at: repair.ready_at,
            collected_at: repair.collected_at,
        },
        customer: PrintCustomer {
            id: customer.id,
            name: customer.name,
            phone: customer.phone,
            email: customer.email,
            address: customer.address,
        },
        device: PrintDevice {
            id: device.id,
            device_type: device.device_type.unwrap_or_default(),
            manufacturer: device.manufacturer,
            model: device.model,
            serial_number: device.serial_number,
        },
        diagnosis,
    })
}

fn map_diagnosis_value(value: Value) -> PrintDiagnosisValue {
    match value {
        Value::Bool(b) => PrintDiagnosisValue::Bool(b),
        Value::String(s) => PrintDiagnosisValue::Text(s),
        Value::Number(n) => PrintDiagnosisValue::Text(n.to_string()),
        Value::Null => PrintDiagnosisValue::Text(String::new()),
        other => PrintDiagnosisValue::Text(other.to_string()),
    }
}
