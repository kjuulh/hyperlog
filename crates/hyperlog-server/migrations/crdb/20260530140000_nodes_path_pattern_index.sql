-- Speed up prefix scans (path LIKE 'p.%') used by the bounded GetView fetch.
CREATE INDEX IF NOT EXISTS idx_nodes_root_path_pattern
    ON nodes (root_id, path text_pattern_ops);
