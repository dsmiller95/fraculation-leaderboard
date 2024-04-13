use std::convert::Infallible;
use std::time::Duration;

use axum::{
    Extension,
    extract::{Path, State},
    Form,
    http::StatusCode, response::{IntoResponse, Response, Sse, sse::Event},
};
use serde_json::json;
use sqlx::query::QueryAs;
use tokio::sync::broadcast::Sender;
use tokio_stream::{Stream, StreamExt as _};
use tokio_stream::wrappers::BroadcastStream;

use crate::{errors::ApiError, router::AppState};
use crate::models::MutationKind;

use super::models::*;
use super::templates;

pub type LeaderboardStream = Sender<LeaderboardUpdate>;

pub async fn home() -> impl IntoResponse {
    templates::HelloTemplate
}

pub async fn stream() -> impl IntoResponse {
    templates::StreamTemplate
}

pub async fn styles() -> Result<impl IntoResponse, ApiError> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/css")
        .body(include_str!("../../templates/styles.css").to_owned())?;

    Ok(response)
}

pub async fn fetch_games(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let games = sqlx::query_as::<_, Game>("SELECT * FROM games")
        .fetch_all(&state.db)
        .await
        .unwrap_or(vec![
            Game { description: "demo game".into(), id: 22},
            Game { description: "demo game two".into(), id: 23}
        ]);

    Ok(templates::Games { games })
}

pub async fn create_game(
    State(state): State<AppState>,
    Form(form): Form<GameNew>,
) -> impl IntoResponse {
    let mock_desc = form.description.clone();
    let game = sqlx::query_as::<_, Game>(
        "INSERT INTO games (description) VALUES ($1) RETURNING id, description",
    )
    .bind(form.description)
    .fetch_one(&state.db)
    .await
    .unwrap_or(
        Game { description: mock_desc, id: 20}
    );

    templates::GameNewTemplate { game }
}

pub async fn leaderboard_home(
    State(state): State<AppState>,
    Path(game_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let game = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id = $1")
        .bind(game_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(
            Game { description: "null game".into(), id: game_id}
        );
    Ok(templates::LeaderboardTemplate { game })
}

pub async fn fetch_leaderboard_entries(
    State(state): State<AppState>,
    Path(game_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let entries = sqlx::query_as::<_, LeaderboardEntry>(
        "SELECT * FROM leaderboard_entries WHERE game_id = $1")
        .bind(game_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or(vec![
            LeaderboardEntry {
                id: 2, score: 33.0, game_id,
                user_name: "user null".into(),
                free_data: "".into(),
            },
            LeaderboardEntry {
                id: 5, score: 33.2412, game_id,
                user_name: "user nulllll".into(),
                free_data: "{\"some\": 281}".into(),
            },
        ]);

    Ok(templates::LeaderboardEntriesTemplate { entries })
}

macro_rules! bind_all {
    // Base case:
    ($i:expr, $x:expr) => (QueryAs::bind($i, $x));
    // `$x` followed by at least one `$y,`
    ($i:expr, $x:expr, $($y:expr),+) => (
        bind_all!(QueryAs::bind($i, $x), $($y),+)
    )
}

pub async fn create_leaderboard_entry(
    State(state): State<AppState>,
    Path(game_id): Path<i32>,
    Extension(tx): Extension<LeaderboardStream>,
    Form(form): Form<LeaderboardEntryNew>,
) -> impl IntoResponse {
    let leaderboard_entry = sqlx::query_as::<_, LeaderboardEntry>(
        "INSERT INTO leaderboard_entries (game_id, score, user_name, free_data) \
        VALUES ($1, $2, $3, $4) \
        RETURNING id, score, game_id, user_name, free_data",
    );
    let leaderboard_entry = bind_all!(leaderboard_entry, game_id, form.score, form.user_name, form.free_data);
    let leaderboard_entry = leaderboard_entry
        .fetch_one(&state.db)
        .await
        .unwrap();

    if tx
        .send(LeaderboardUpdate {
            mutation_kind: MutationKind::Create,
            id: leaderboard_entry.id,
        })
        .is_err()
    {
        eprintln!(
            "Record with ID {} was created but nobody's listening to the stream!",
            leaderboard_entry.id
        );
    }

    templates::LeaderboardEntryNewTemplate { entry: leaderboard_entry }
}

// TODO: a unique stream per game?
pub async fn handle_stream(
    Extension(tx): Extension<LeaderboardStream>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = tx.subscribe();

    let stream = BroadcastStream::new(rx);

    Sse::new(
        stream
            .map(|msg| {
                let msg = msg.unwrap();
                let json = format!("<div>{}</div>", json!(msg));
                Event::default().data(json)
            })
            .map(Ok),
    )
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(600))
                .text("keep-alive-text"),
        )
}
