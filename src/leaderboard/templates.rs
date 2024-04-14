use super::models;
use askama::Template;

#[derive(Template)]
#[template(path = "leaderboard/index.html")]
pub struct HelloTemplate;

#[derive(Template)]
#[template(path = "leaderboard/stream_page.html")]
pub struct StreamTemplate;

#[derive(Template)]
#[template(path = "leaderboard/games.html")]
pub struct Games {
    pub games: Vec<models::Game>,
}

#[derive(Template)]
#[template(path = "leaderboard/game_row.html")]
pub struct GameNewTemplate {
    pub game: models::Game,
}

#[derive(Template)]
#[template(path = "leaderboard/game.html")]
pub struct GameTemplate {
    pub game: models::Game,
}


#[derive(Template)]
#[template(path = "leaderboard/game_entries.html")]
pub struct LeaderboardEntriesTemplate {
    pub entries: Vec<models::LeaderboardEntry>,
}

#[derive(Template)]
#[template(path = "leaderboard/game_entry_row.html")]
pub struct LeaderboardEntryNewTemplate {
    pub entry: models::LeaderboardEntry,
}
