CREATE TABLE IF NOT EXISTS games (
	id SERIAL PRIMARY KEY,
	description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS leaderboard_entries (
    id SERIAL PRIMARY KEY,
    game_id INTEGER REFERENCES games(id),
    score FLOAT,
    user_name TEXT NOT NULL,
    free_data TEXT NOT NULL
);
