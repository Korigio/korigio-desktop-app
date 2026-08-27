-- Phase 7 search indexes (B-tree; LIKE still scan-limited for mid-string matches).

CREATE INDEX idx_customers_phone ON customers (phone);
CREATE INDEX idx_customers_email ON customers (email);
CREATE INDEX idx_devices_manufacturer ON devices (manufacturer);
CREATE INDEX idx_devices_model ON devices (model);
CREATE INDEX idx_repairs_repair_number ON repairs (repair_number);
CREATE INDEX idx_repairs_reported_problem ON repairs (reported_problem);
