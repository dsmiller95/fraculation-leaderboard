use super::models;
use askama::Template;

#[derive(Template)]
#[template(path = "leaderboard/index.html")]
pub struct HelloTemplate;

#[derive(Template)]
#[template(path = "leaderboard/stream.html")]
pub struct StreamTemplate;

#[derive(Template)]
#[template(path = "leaderboard/games.html")]
pub struct Games {
    pub games: Vec<models::Game>,
}

#[derive(Template)]
#[template(path = "leaderboard/game.html")]
pub struct GameNewTemplate {
    pub game: models::Game,
}

#[derive(Template)]
#[template(path = "leaderboard/leaderboard.html")]
pub struct LeaderboardTemplate {
    pub game: models::Game,
}


#[derive(Template)]
#[template(path = "leaderboard/leaderboard_entries.html")]
pub struct LeaderboardEntriesTemplate {
    pub entries: Vec<models::LeaderboardEntry>,
}

#[derive(Template)]
#[template(path = "leaderboard/leaderboard_entry.html")]
pub struct LeaderboardEntryNewTemplate {
    pub entry: models::LeaderboardEntry,
}
