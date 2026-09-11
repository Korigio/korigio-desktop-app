-- Intake estimate list price + discount (basis points) on repairs.
-- Post-discount net remains estimate_base_cents; diagnosis overwrites clear these to NULL.

ALTER TABLE repairs ADD COLUMN estimate_list_cents INTEGER;
ALTER TABLE repairs ADD COLUMN estimate_discount_bps INTEGER;
