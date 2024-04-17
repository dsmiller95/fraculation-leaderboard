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
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_rapidoc::RapiDoc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

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
    modifiers(&JsonContentTypeAddon),
    tags(
        (name = "leaderboard", description = "Game Leaderboard management API")
    )
)]
struct ApiDoc;


struct JsonContentTypeAddon;

impl Modify for JsonContentTypeAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
            )
        }
    }
}

pub fn init_router(db: PgPool) -> Router {
    let state = AppState { db };

    let mut router = Router::new()
        .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", ApiDoc::openapi()).path("/rapidoc"))
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
