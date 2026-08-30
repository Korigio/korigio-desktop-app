-- Phase 17: clean-slate UUID v7 PKs, HLC, staff/team, sync log.
-- DROPs all domain tables (NOT schema_migrations) and recreates the full schema.
-- Foreign keys are disabled by the migration runner for this rebuild.

DROP TABLE IF EXISTS repair_documents;
DROP TABLE IF EXISTS repair_images;
DROP TABLE IF EXISTS repair_diagnosis;
DROP TABLE IF EXISTS repairs;
DROP TABLE IF EXISTS devices;
DROP TABLE IF EXISTS customers;
DROP TABLE IF EXISTS companies;
DROP TABLE IF EXISTS diagnosis_templates;
DROP TABLE IF EXISTS repair_number_sequences;
DROP TABLE IF EXISTS settings;
DROP TABLE IF EXISTS staff;
DROP TABLE IF EXISTS team_invites;
DROP TABLE IF EXISTS team_devices;
DROP TABLE IF EXISTS teams;
DROP TABLE IF EXISTS content_blobs;
DROP TABLE IF EXISTS sync_changes;
DROP TABLE IF EXISTS presence;
DROP TABLE IF EXISTS sync_peer_cursors;
DROP TABLE IF EXISTS local_identity;

CREATE TABLE settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE local_identity (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id = 1),
    device_id TEXT NOT NULL,
    device_code TEXT,
    device_name TEXT NOT NULL,
    team_id TEXT,
    team_psk TEXT,
    current_staff_id TEXT,
    session_started_at TEXT,
    hlc_wall_ms INTEGER NOT NULL DEFAULT 0,
    hlc_counter INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE teams (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    updated_by_staff_id TEXT,
    deleted_at TEXT
);

CREATE TABLE staff (
    id TEXT PRIMARY KEY NOT NULL,
    team_id TEXT REFERENCES teams (id),
    name TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('admin', 'staff')),
    pin_salt TEXT NOT NULL,
    pin_hash TEXT NOT NULL,
    deactivated_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    updated_by_staff_id TEXT,
    deleted_at TEXT
);

CREATE TABLE team_devices (
    id TEXT PRIMARY KEY NOT NULL,
    team_id TEXT NOT NULL REFERENCES teams (id),
    device_code TEXT NOT NULL CHECK (device_code GLOB '[A-Z][A-Z]'),
    device_name TEXT NOT NULL,
    joined_at TEXT NOT NULL,
    removed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    updated_by_staff_id TEXT,
    deleted_at TEXT,
    UNIQUE (team_id, device_code)
);

CREATE TABLE team_invites (
    id TEXT PRIMARY KEY NOT NULL,
    team_id TEXT NOT NULL REFERENCES teams (id),
    code_hash TEXT NOT NULL,
    created_by_staff_id TEXT NOT NULL REFERENCES staff (id),
    expires_at TEXT NOT NULL,
    revoked_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    updated_by_staff_id TEXT,
    deleted_at TEXT
);

CREATE TABLE companies (
    id TEXT PRIMARY KEY NOT NULL,
    legal_name TEXT NOT NULL,
    trade_name TEXT,
    tax_id TEXT,
    address TEXT,
    phone TEXT,
    email TEXT,
    website TEXT,
    logo_path TEXT,
    logo_content_hash TEXT,
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    archived_at TEXT,
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    updated_by_staff_id TEXT,
    deleted_at TEXT
);

CREATE TABLE customers (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    phone TEXT,
    email TEXT,
    address TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    archived_at TEXT,
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    updated_by_staff_id TEXT,
    deleted_at TEXT
);

CREATE TABLE devices (
    id TEXT PRIMARY KEY NOT NULL,
    customer_id TEXT NOT NULL REFERENCES customers (id),
    device_type TEXT,
    manufacturer TEXT,
    model TEXT,
    serial_number TEXT,
    accessories TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    archived_at TEXT,
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    updated_by_staff_id TEXT,
    deleted_at TEXT
);

CREATE TABLE diagnosis_templates (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    body_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    updated_by_staff_id TEXT,
    deleted_at TEXT
);

CREATE TABLE repairs (
    id TEXT PRIMARY KEY NOT NULL,
    repair_number TEXT NOT NULL UNIQUE,
    customer_id TEXT NOT NULL REFERENCES customers (id),
    device_id TEXT NOT NULL REFERENCES devices (id),
    company_id TEXT REFERENCES companies (id),
    assigned_to_staff_id TEXT REFERENCES staff (id),
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
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    updated_by_staff_id TEXT,
    deleted_at TEXT,
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

CREATE TABLE repair_images (
    id TEXT PRIMARY KEY NOT NULL,
    repair_id TEXT NOT NULL REFERENCES repairs (id),
    original_path TEXT NOT NULL,
    thumb_path TEXT,
    content_hash TEXT NOT NULL,
    caption TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    updated_by_staff_id TEXT,
    deleted_at TEXT
);

CREATE TABLE repair_diagnosis (
    id TEXT PRIMARY KEY NOT NULL,
    repair_id TEXT NOT NULL REFERENCES repairs (id),
    template_id TEXT REFERENCES diagnosis_templates (id),
    result_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    updated_by_staff_id TEXT,
    deleted_at TEXT
);

CREATE TABLE repair_documents (
    id TEXT PRIMARY KEY NOT NULL,
    repair_id TEXT NOT NULL REFERENCES repairs (id) ON DELETE CASCADE,
    document_type TEXT NOT NULL CHECK (document_type IN ('entrance_signed', 'diagnosis_signed', 'summary_signed')),
    file_path TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    updated_by_staff_id TEXT,
    deleted_at TEXT,
    UNIQUE (repair_id, document_type)
);

CREATE TABLE content_blobs (
    content_hash TEXT PRIMARY KEY NOT NULL,
    byte_size INTEGER NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('image', 'thumb', 'document', 'logo')),
    created_at TEXT NOT NULL
);

CREATE TABLE sync_changes (
    id TEXT PRIMARY KEY NOT NULL,
    entity_table TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    op TEXT NOT NULL CHECK (op IN ('upsert', 'delete')),
    payload_json TEXT NOT NULL,
    hlc_wall_ms INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE presence (
    id TEXT PRIMARY KEY NOT NULL,
    device_id TEXT NOT NULL UNIQUE,
    staff_id TEXT,
    last_seen_at TEXT NOT NULL
);

CREATE TABLE sync_peer_cursors (
    peer_device_id TEXT PRIMARY KEY NOT NULL,
    wall INTEGER NOT NULL,
    counter INTEGER NOT NULL,
    origin_device_id TEXT NOT NULL
);

CREATE TABLE repair_number_sequences (
    year INTEGER NOT NULL,
    device_code TEXT NOT NULL,
    last_value INTEGER NOT NULL,
    PRIMARY KEY (year, device_code)
);

CREATE UNIQUE INDEX idx_repair_diagnosis_repair_id ON repair_diagnosis (repair_id);

CREATE INDEX idx_devices_customer_id ON devices (customer_id);
CREATE INDEX idx_devices_serial_number ON devices (serial_number);
CREATE INDEX idx_devices_manufacturer ON devices (manufacturer);
CREATE INDEX idx_devices_model ON devices (model);

CREATE INDEX idx_repairs_customer_id ON repairs (customer_id);
CREATE INDEX idx_repairs_device_id ON repairs (device_id);
CREATE INDEX idx_repairs_status ON repairs (status);
CREATE INDEX idx_repairs_received_at ON repairs (received_at);
CREATE INDEX idx_repairs_company_id ON repairs (company_id);
CREATE INDEX idx_repairs_repair_number ON repairs (repair_number);
CREATE INDEX idx_repairs_reported_problem ON repairs (reported_problem);
CREATE INDEX idx_repairs_assigned_to_staff_id ON repairs (assigned_to_staff_id);

CREATE INDEX idx_repair_images_repair_id ON repair_images (repair_id);
CREATE INDEX idx_repair_images_content_hash ON repair_images (content_hash);
CREATE INDEX idx_repair_documents_repair_id ON repair_documents (repair_id);
CREATE INDEX idx_repair_documents_content_hash ON repair_documents (content_hash);

CREATE INDEX idx_customers_name ON customers (name);
CREATE INDEX idx_customers_archived_at ON customers (archived_at);
CREATE INDEX idx_customers_phone ON customers (phone);
CREATE INDEX idx_customers_email ON customers (email);

CREATE INDEX idx_companies_legal_name ON companies (legal_name);
CREATE INDEX idx_companies_archived_at ON companies (archived_at);
CREATE INDEX idx_companies_is_default ON companies (is_default);

CREATE INDEX idx_diagnosis_templates_name ON diagnosis_templates (name);

CREATE INDEX idx_staff_team_id ON staff (team_id);
CREATE INDEX idx_staff_deactivated_at ON staff (deactivated_at);
CREATE INDEX idx_team_devices_team_id ON team_devices (team_id);
CREATE INDEX idx_team_invites_team_id ON team_invites (team_id);

CREATE INDEX idx_sync_changes_hlc ON sync_changes (hlc_wall_ms, hlc_counter, origin_device_id);
CREATE INDEX idx_sync_changes_entity ON sync_changes (entity_table, entity_id);
