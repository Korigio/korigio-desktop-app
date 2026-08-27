-- Companies (shop profiles) + repairs.company_id FK.
-- company_id is nullable for migrate safety on leftover rows; create_repair requires it.

PRAGMA foreign_keys = ON;

CREATE TABLE companies (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    legal_name TEXT NOT NULL,
    trade_name TEXT,
    tax_id TEXT,
    address TEXT,
    phone TEXT,
    email TEXT,
    website TEXT,
    logo_path TEXT,
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    archived_at TEXT
);

CREATE INDEX idx_companies_legal_name ON companies (legal_name);
CREATE INDEX idx_companies_archived_at ON companies (archived_at);
CREATE INDEX idx_companies_is_default ON companies (is_default);

ALTER TABLE repairs ADD COLUMN company_id INTEGER REFERENCES companies (id);

CREATE INDEX idx_repairs_company_id ON repairs (company_id);
