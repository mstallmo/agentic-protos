-- Add statistics columns to the counters table
ALTER TABLE counters ADD COLUMN total_increments INTEGER NOT NULL DEFAULT 0;
ALTER TABLE counters ADD COLUMN average_increment REAL NOT NULL DEFAULT 0.0;
ALTER TABLE counters ADD COLUMN highest_value INTEGER NOT NULL DEFAULT 0;
ALTER TABLE counters ADD COLUMN description TEXT;

-- Create a view to easily access counter statistics
CREATE VIEW counter_stats AS
SELECT 
    id,
    value AS current_value,
    total_increments,
    average_increment,
    highest_value,
    created_at,
    updated_at,
    description
FROM counters;

-- Create a trigger to update statistics when a counter is incremented
CREATE TRIGGER IF NOT EXISTS update_counter_stats
AFTER UPDATE OF value ON counters
WHEN NEW.value > OLD.value
BEGIN
    UPDATE counters SET 
        total_increments = total_increments + 1,
        average_increment = (OLD.average_increment * OLD.total_increments + (NEW.value - OLD.value)) / (OLD.total_increments + 1),
        highest_value = MAX(highest_value, NEW.value)
    WHERE id = NEW.id;
END;