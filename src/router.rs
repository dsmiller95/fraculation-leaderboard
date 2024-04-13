use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    routing::{delete, get},
    Extension, Router,
};
use sqlx::PgPool;

use tokio::sync::broadcast::channel;
use crate::todo;
use crate::leaderboard;


#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct RootHelloTemplate;

pub async fn root_home() -> impl IntoResponse { RootHelloTemplate }

pub fn init_router(db: PgPool) -> Router {
    let state = AppState { db };

    let mut router = Router::new()
        .route("/", get(root_home))
        ;
    {
        use todo::routes::*;
        use todo::models::TodoUpdate;

        let (tx, _rx) = channel::<TodoUpdate>(10);
        let update_stream: TodosStream = tx;
        router = router
            .route("/todo", get(home))
            .route("/todo/stream", get(stream))
            .route("/todo/styles.css", get(styles))
            .route("/todo/todos", get(fetch_todos).post(create_todo))
            .route("/todo/todos/:id", delete(delete_todo))
            .route("/todo/todos/stream", get(handle_stream))
            .layer(Extension(update_stream))
    }
    {
        use leaderboard::routes::*;
        use leaderboard::models::LeaderboardUpdate;

        let (tx, _rx) = channel::<LeaderboardUpdate>(10);
        let update_stream: LeaderboardStream = tx;
        router = router
            .route("/leaderboard", get(home))
            .route("/leaderboard/stream", get(stream))
            .route("/leaderboard/styles.css", get(styles))
            .route("/leaderboard/games", get(fetch_games).post(create_game))
            .route("/leaderboard/:id", get(leaderboard_home))
            .route("/leaderboard/games/:id", get(fetch_leaderboard_entries).post(create_leaderboard_entry))
            .route("/leaderboard/games/stream", get(handle_stream))
            .layer(Extension(update_stream))
    }

    router
        .with_state(state)
}
