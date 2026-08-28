-- Estimate snapshot columns on repairs (tax-exclusive cents).
-- Settings keys tax_rate_percent / currency use the existing settings table (no schema).

ALTER TABLE repairs ADD COLUMN estimate_base_cents INTEGER;
ALTER TABLE repairs ADD COLUMN estimate_tax_rate_bps INTEGER;
ALTER TABLE repairs ADD COLUMN estimate_tax_cents INTEGER;
ALTER TABLE repairs ADD COLUMN estimate_gross_cents INTEGER;
