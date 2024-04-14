-- Create a new ENUM type for GameScoreSortMode
CREATE TYPE GameScoreSortMode AS ENUM ('HigherIsBetter', 'LesserIsBetter');

-- Add a new column to the games table using the GameScoreSortMode type
ALTER TABLE games ADD COLUMN score_sort_mode GameScoreSortMode;
UPDATE games SET score_sort_mode = 'HigherIsBetter' WHERE score_sort_mode IS NULL;