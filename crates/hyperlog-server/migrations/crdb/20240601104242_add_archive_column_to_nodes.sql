-- Add migration script here

ALTER TABLE nodes ADD COLUMN status VARCHAR(20) DEFAULT 'active' NOT NULL;
