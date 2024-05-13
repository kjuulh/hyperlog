-- Add migration script here

CREATE TABLE roots (
    id UUID NOT NULL PRIMARY KEY,
    root_name VARCHAR(255) UNIQUE NOT NULL
);

CREATE TABLE nodes (
    id UUID NOT NULL PRIMARY KEY,
    root_id UUID NOT NULL,
    path VARCHAR NOT NULL,
    item_type VARCHAR NOT NULL,
    item_content JSONB
);
