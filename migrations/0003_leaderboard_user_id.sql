-- Add a new column to the leaderboard_entries table for the uuid of the user. set everything to random ids
ALTER TABLE leaderboard_entries
    ADD COLUMN user_id UUID NOT NULL DEFAULT gen_random_uuid();

-- remove the default value, to enforce setting uuid from application side
ALTER TABLE leaderboard_entries
    ALTER COLUMN user_id DROP DEFAULT;