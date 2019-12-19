-- removes the deltas.features column
ALTER TABLE deltas
DROP COLUMN IF EXISTS features;
