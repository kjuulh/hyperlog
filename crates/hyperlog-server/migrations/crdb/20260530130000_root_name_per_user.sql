-- Workspaces: root names are unique PER USER, not globally, so two users can
-- both have a "personal" workspace. (Legacy rows have user_id NULL.)
ALTER TABLE roots DROP CONSTRAINT IF EXISTS roots_root_name_key;
CREATE UNIQUE INDEX IF NOT EXISTS idx_roots_user_name ON roots(user_id, root_name);
