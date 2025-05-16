-- Create counters table
CREATE TABLE IF NOT EXISTS counters (
    id TEXT PRIMARY KEY,
    value INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create a trigger to automatically update the updated_at timestamp
CREATE TRIGGER IF NOT EXISTS update_counters_timestamp
AFTER UPDATE ON counters
BEGIN
    UPDATE counters SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- Create an index for faster lookups by id (though with a primary key this is redundant)
CREATE INDEX IF NOT EXISTS idx_counters_id ON counters(id);

-- Add a default counter if needed (commented out as this should be handled by the application code)
-- INSERT OR IGNORE INTO counters (id, value) VALUES ('main_counter', 0);