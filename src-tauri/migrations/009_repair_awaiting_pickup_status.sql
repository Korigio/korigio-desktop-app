-- Allow awaiting_pickup between ready and collected (signed summary step).
-- Note: foreign_keys are disabled by the migration runner for this rebuild.
CREATE TABLE repairs_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    repair_number TEXT NOT NULL UNIQUE,
    customer_id INTEGER NOT NULL REFERENCES customers (id),
    device_id INTEGER NOT NULL REFERENCES devices (id),
    company_id INTEGER REFERENCES companies (id),
    status TEXT NOT NULL,
    received_at TEXT NOT NULL,
    reported_problem TEXT,
    accessories_received TEXT,
    device_condition TEXT,
    diagnosis_notes TEXT,
    work_performed TEXT,
    notes TEXT,
    expected_pickup_at TEXT,
    estimate_base_cents INTEGER,
    estimate_tax_rate_bps INTEGER,
    estimate_tax_cents INTEGER,
    estimate_gross_cents INTEGER,
    ready_at TEXT,
    collected_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    archived_at TEXT,
    CHECK (
        status IN (
            'received',
            'diagnosis',
            'waiting_customer',
            'waiting_part',
            'in_repair',
            'ready',
            'awaiting_pickup',
            'collected',
            'cancelled'
        )
    )
);

INSERT INTO repairs_new (
    id,
    repair_number,
    customer_id,
    device_id,
    company_id,
    status,
    received_at,
    reported_problem,
    accessories_received,
    device_condition,
    diagnosis_notes,
    work_performed,
    notes,
    expected_pickup_at,
    estimate_base_cents,
    estimate_tax_rate_bps,
    estimate_tax_cents,
    estimate_gross_cents,
    ready_at,
    collected_at,
    created_at,
    updated_at,
    archived_at
)
SELECT
    id,
    repair_number,
    customer_id,
    device_id,
    company_id,
    status,
    received_at,
    reported_problem,
    accessories_received,
    device_condition,
    diagnosis_notes,
    work_performed,
    notes,
    expected_pickup_at,
    estimate_base_cents,
    estimate_tax_rate_bps,
    estimate_tax_cents,
    estimate_gross_cents,
    ready_at,
    collected_at,
    created_at,
    updated_at,
    archived_at
FROM repairs;

DROP TABLE repairs;
ALTER TABLE repairs_new RENAME TO repairs;

CREATE INDEX idx_repairs_customer_id ON repairs (customer_id);
CREATE INDEX idx_repairs_device_id ON repairs (device_id);
CREATE INDEX idx_repairs_status ON repairs (status);
CREATE INDEX idx_repairs_received_at ON repairs (received_at);
CREATE INDEX idx_repairs_company_id ON repairs (company_id);
CREATE INDEX idx_repairs_repair_number ON repairs (repair_number);
CREATE INDEX idx_repairs_reported_problem ON repairs (reported_problem);
