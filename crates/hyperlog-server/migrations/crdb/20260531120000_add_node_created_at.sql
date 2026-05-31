-- Project-management metadata: a real creation timestamp per node. Existing
-- rows get now() at migration time; new rows get their true insert time. due
-- date + links live in item_content JSON (client-owned); created_at is DB-owned
-- so it survives every update without preserve logic.
ALTER TABLE nodes ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT now();
