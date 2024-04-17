use utoipa::{OpenApi};

use crate::leaderboard;

pub fn gen_my_openapi() -> utoipa::openapi::OpenApi {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            leaderboard::routes::get_games,
            leaderboard::routes::create_game,
            leaderboard::routes::get_game,
            leaderboard::routes::get_game_entries,
            leaderboard::routes::get_user_game_entry,
            leaderboard::routes::create_game_entry,
        ),
        components(
            schemas(
                leaderboard::models::Game, leaderboard::models::GameNew, leaderboard::models::GameScoreSortMode,
                leaderboard::models::LeaderboardEntry, leaderboard::models::LeaderboardEntryNew, leaderboard::models::LeaderboardUpdate,
            )
        ),
        modifiers(),
        tags(
            (name = "leaderboard", description = "Game Leaderboard management API")
        )
    )]
    struct ApiDoc;


    ApiDoc::openapi()
}