-- Phase 2 initial schema (Repair Manager)
-- Timestamps stored as ISO-8601 text (UTC).
-- schema_migrations is owned by the migration runner (not this file).

PRAGMA foreign_keys = ON;

CREATE TABLE customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    phone TEXT,
    email TEXT,
    address TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    archived_at TEXT
);

CREATE TABLE devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    customer_id INTEGER NOT NULL REFERENCES customers (id),
    device_type TEXT,
    manufacturer TEXT,
    model TEXT,
    serial_number TEXT,
    accessories TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    archived_at TEXT
);

CREATE TABLE repairs (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    repair_number TEXT NOT NULL UNIQUE,
    customer_id INTEGER NOT NULL REFERENCES customers (id),
    device_id INTEGER NOT NULL REFERENCES devices (id),
    status TEXT NOT NULL,
    received_at TEXT NOT NULL,
    reported_problem TEXT,
    accessories_received TEXT,
    device_condition TEXT,
    diagnosis_notes TEXT,
    work_performed TEXT,
    notes TEXT,
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
            'collected',
            'cancelled'
        )
    )
);

CREATE TABLE repair_number_sequences (
    year INTEGER PRIMARY KEY NOT NULL,
    last_value INTEGER NOT NULL
);

CREATE TABLE repair_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    repair_id INTEGER NOT NULL REFERENCES repairs (id),
    original_path TEXT NOT NULL,
    thumb_path TEXT,
    caption TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE diagnosis_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    body_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE repair_diagnosis (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    repair_id INTEGER NOT NULL REFERENCES repairs (id),
    template_id INTEGER REFERENCES diagnosis_templates (id),
    result_json TEXT NOT NULL
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE INDEX idx_devices_customer_id ON devices (customer_id);
CREATE INDEX idx_devices_serial_number ON devices (serial_number);
CREATE INDEX idx_repairs_customer_id ON repairs (customer_id);
CREATE INDEX idx_repairs_device_id ON repairs (device_id);
CREATE INDEX idx_repairs_status ON repairs (status);
CREATE INDEX idx_repairs_received_at ON repairs (received_at);
CREATE INDEX idx_repair_images_repair_id ON repair_images (repair_id);
CREATE INDEX idx_customers_name ON customers (name);
CREATE INDEX idx_customers_archived_at ON customers (archived_at);
