use std::convert::Infallible;
use std::time::Duration;

use axum::{Extension, extract::{Path, State}, Json, response::{IntoResponse, Sse, sse::Event}};
use serde_json::json;
use sqlx::query::QueryAs;
use tokio::sync::broadcast::Sender;
use tokio_stream::{Stream, StreamExt as _};
use tokio_stream::wrappers::BroadcastStream;

use crate::{errors::ApiError, router::AppState};
use crate::hetero_req_resp::{JsonOrForm, AcceptType};
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

pub async fn fetch_games(
    accept_type: AcceptType,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let games = sqlx::query_as::<_, Game>("SELECT * FROM games")
        .fetch_all(&state.db)
        .await?;

    Ok(match accept_type {
        AcceptType::HTMX => templates::Games { games }.into_response(),
        AcceptType::JSON => Json(games).into_response()
    })
}

pub async fn create_game(
    accept_type: AcceptType,
    State(state): State<AppState>,
    JsonOrForm(request): JsonOrForm<GameNew>,
) -> Result<impl IntoResponse, ApiError> {
    let game = sqlx::query_as::<_, Game>(
        "INSERT INTO games (description) VALUES ($1) RETURNING id, description",
    )
    .bind(request.description)
    .fetch_one(&state.db)
    .await?;

    Ok(match accept_type {
        AcceptType::HTMX => templates::GameNewTemplate { game }.into_response(),
        AcceptType::JSON => Json(game).into_response()
    })
}

pub async fn game_home(
    accept_type: AcceptType,
    State(state): State<AppState>,
    Path(game_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let game = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id = $1")
        .bind(game_id)
        .fetch_one(&state.db)
        .await?;

    Ok(match accept_type {
        AcceptType::HTMX => templates::GameTemplate { game }.into_response(),
        AcceptType::JSON => Json(game).into_response()
    })
}

pub async fn fetch_leaderboard_entries(
    accept_type: AcceptType,
    State(state): State<AppState>,
    Path(game_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let entries = sqlx::query_as::<_, LeaderboardEntry>(
        "SELECT * \
            FROM leaderboard_entries \
            WHERE game_id = $1 \
            ORDER BY score desc \
            LIMIT 10;")
        .bind(game_id)
        .fetch_all(&state.db)
        .await?;

    Ok(match accept_type {
        AcceptType::HTMX => templates::LeaderboardEntriesTemplate { entries }.into_response(),
        AcceptType::JSON => Json(entries).into_response()
    })
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
    accept_type: AcceptType,
    State(state): State<AppState>,
    Path(game_id): Path<i32>,
    Extension(tx): Extension<LeaderboardStream>,
    JsonOrForm(request): JsonOrForm<LeaderboardEntryNew>,
) -> Result<impl IntoResponse, ApiError> {
    let leaderboard_entry = sqlx::query_as::<_, LeaderboardEntry>(
        "INSERT INTO leaderboard_entries (game_id, score, user_name, free_data) \
        VALUES ($1, $2, $3, $4) \
        RETURNING id, score, game_id, user_name, free_data",
    );
    let leaderboard_entry = bind_all!(leaderboard_entry, game_id, request.score, request.user_name, request.free_data);
    let leaderboard_entry = leaderboard_entry
        .fetch_one(&state.db)
        .await?;

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

    Ok(match accept_type {
        AcceptType::HTMX => templates::LeaderboardEntryNewTemplate { entry: leaderboard_entry }.into_response(),
        AcceptType::JSON => Json(leaderboard_entry).into_response()
    })
}

// TODO: a unique stream per game?
pub async fn handle_stream(
    accept_type: AcceptType,
    Extension(tx): Extension<LeaderboardStream>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = tx.subscribe();

    let stream = BroadcastStream::new(rx);

    Sse::new(
        stream
            .map(move |msg| {
                let msg = msg.unwrap();
                let json = json!(msg);
                let message = match accept_type {
                    AcceptType::HTMX => format!("<div>{}</div>", json),
                    AcceptType::JSON => json.to_string()
                };
                Event::default().data(message)
            })
            .map(Ok),
    )
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(600))
                .text("keep-alive-text"),
        )
}
