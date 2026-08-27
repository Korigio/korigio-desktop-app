CREATE UNIQUE INDEX idx_repair_diagnosis_repair_id ON repair_diagnosis (repair_id);
CREATE INDEX idx_diagnosis_templates_name ON diagnosis_templates (name);
