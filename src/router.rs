use askama::Template;
use askama_axum::{IntoResponse, Response};
use axum::http::header::{ACCEPT, CONTENT_TYPE};
use axum::http::{Method, StatusCode};
use axum::{
    routing::{delete, get},
    Extension, Router,
};
use sqlx::PgPool;

use crate::errors::ApiError;
use crate::leaderboard;
use crate::todo;
use tokio::sync::broadcast::channel;
use tower_http::cors::{Any, CorsLayer};
use utoipa_rapidoc::RapiDoc;
use crate::app_state::AppState;
use crate::openapi::gen_my_openapi;


#[derive(Template)]
#[template(path = "index.html")]
pub struct RootHelloTemplate;

pub async fn root_home() -> impl IntoResponse {
    RootHelloTemplate
}
pub async fn styles() -> Result<impl IntoResponse, ApiError> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/css")
        .body(include_str!("../templates/styles.css").to_owned())?;

    Ok(response)
}

pub fn init_router(db: PgPool) -> Router {
    let state = AppState { db };
    let open_api = gen_my_openapi();
    let mut router = Router::new()
        .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", open_api).path("/rapidoc"))
        .route("/", get(root_home))
        .route("/styles.css", get(styles));
    {
        use todo::models::TodoUpdate;
        use todo::routes::*;

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
        use leaderboard::models::LeaderboardUpdate;
        use leaderboard::routes::*;

        let (tx, _rx) = channel::<LeaderboardUpdate>(10);
        let update_stream: LeaderboardStream = tx;
        router = router
            .route("/leaderboard", get(home))
            .route("/leaderboard/stream_page", get(stream))
            .route("/leaderboard/stream", get(handle_stream))
            .route("/leaderboard/styles.css", get(styles))

            .route("/leaderboard/games", get(get_games).post(create_game))
            .route("/leaderboard/games/:game_id", get(get_game))
            .route(
                "/leaderboard/games/:game_id/entries",
                get(get_game_entries).post(create_game_entry),
            )
            .route(
                "/leaderboard/users/:user_id/games/:game_id/entries",
                get(get_user_game_entry),
            )
            .layer(Extension(update_stream))
    }
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // Accept defines whether to send back json or html. content type defines form data vs json data
        .allow_headers([ACCEPT, CONTENT_TYPE])
        // allow requests from any origin
        .allow_origin(Any);

    router
        .layer(cors)
        .with_state(state)
}
