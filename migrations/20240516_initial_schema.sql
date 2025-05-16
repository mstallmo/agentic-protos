-- Migration script for initializing SQLite schema
-- Migration: 20240516_initial_schema

-- Create counters table
CREATE TABLE IF NOT EXISTS counters (
    id TEXT PRIMARY KEY,
    value INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create a trigger to update the updated_at timestamp
CREATE TRIGGER IF NOT EXISTS update_counters_timestamp
AFTER UPDATE ON counters
BEGIN
    UPDATE counters SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- Insert default counter if it doesn't exist
INSERT OR IGNORE INTO counters (id, value) VALUES ('main_counter', 0);