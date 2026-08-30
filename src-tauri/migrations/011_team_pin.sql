-- Team join PIN: SHA-256 hex on the shared team row; plaintext only on this PC.
ALTER TABLE teams ADD COLUMN pin_hash TEXT;
ALTER TABLE local_identity ADD COLUMN team_pin TEXT;
