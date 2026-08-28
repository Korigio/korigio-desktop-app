-- Signed documents (entrance, diagnosis, summary) — one row per type per repair.
CREATE TABLE repair_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repair_id INTEGER NOT NULL REFERENCES repairs(id) ON DELETE CASCADE,
    document_type TEXT NOT NULL CHECK(document_type IN ('entrance_signed', 'diagnosis_signed', 'summary_signed')),
    file_path TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(repair_id, document_type)
);

CREATE INDEX idx_repair_documents_repair_id ON repair_documents(repair_id);

INSERT INTO repair_documents (repair_id, document_type, file_path, original_filename, created_at, updated_at)
SELECT id, 'diagnosis_signed', signed_diagnosis_path, COALESCE(signed_diagnosis_filename, 'signed-diagnosis'), updated_at, updated_at
FROM repairs
WHERE signed_diagnosis_path IS NOT NULL AND TRIM(signed_diagnosis_path) != '';

ALTER TABLE repairs DROP COLUMN signed_diagnosis_path;
ALTER TABLE repairs DROP COLUMN signed_diagnosis_filename;
