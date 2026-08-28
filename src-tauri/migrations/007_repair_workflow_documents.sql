-- Signed customer diagnosis document + workflow document storage paths.
ALTER TABLE repairs ADD COLUMN signed_diagnosis_path TEXT NULL;
ALTER TABLE repairs ADD COLUMN signed_diagnosis_filename TEXT NULL;
