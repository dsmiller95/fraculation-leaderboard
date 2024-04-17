use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    response::{sse::Event, IntoResponse, Sse},
    Extension, Json,
};
use serde_json::json;
use sqlx::query::QueryAs;
use sqlx::types::Uuid;
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt as _};

use super::models::*;
use super::templates;
use crate::hetero_req_resp::{AcceptType, JsonOrForm};
use crate::models::MutationKind;
use crate::{errors::ApiError, router::AppState};

pub type LeaderboardStream = Sender<LeaderboardUpdate>;

pub async fn home() -> impl IntoResponse {
    templates::HelloTemplate
}

pub async fn stream() -> impl IntoResponse {
    templates::StreamTemplate
}

pub async fn get_games(
    accept_type: AcceptType,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let games = sqlx::query_as::<_, Game>("SELECT * FROM games")
        .fetch_all(&state.db)
        .await?;

    Ok(match accept_type {
        AcceptType::HTMX => templates::Games { games }.into_response(),
        AcceptType::JSON => Json(games).into_response(),
    })
}

pub async fn create_game(
    accept_type: AcceptType,
    State(state): State<AppState>,
    JsonOrForm(request): JsonOrForm<GameNew>,
) -> Result<impl IntoResponse, ApiError> {
    let game = sqlx::query_as::<_, Game>(
        "INSERT INTO games (description, score_sort_mode) VALUES ($1, $2) RETURNING id, description, score_sort_mode",
    )
        .bind(request.description)
        .bind(request.score_sort_mode.unwrap_or(GameScoreSortMode::HigherIsBetter))
    .fetch_one(&state.db)
    .await?;

    Ok(match accept_type {
        AcceptType::HTMX => templates::GameNewTemplate { game }.into_response(),
        AcceptType::JSON => Json(game).into_response(),
    })
}

pub async fn get_game(
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
        AcceptType::JSON => Json(game).into_response(),
    })
}

pub async fn get_game_entries(
    accept_type: AcceptType,
    State(state): State<AppState>,
    Path(game_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let game = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE id = $1")
        .bind(game_id)
        .fetch_one(&state.db)
        .await?;
    let ordering = match game.score_sort_mode {
        GameScoreSortMode::HigherIsBetter => "DESC",
        GameScoreSortMode::LesserIsBetter => "ASC",
    };
    let sql = format!(
        "SELECT * \
            FROM leaderboard_entries \
            WHERE game_id = $1 \
            ORDER BY score {} \
            LIMIT 10;",
        ordering
    );
    let entries = sqlx::query_as::<_, LeaderboardEntry>(sql.as_str())
        .bind(game_id)
        .fetch_all(&state.db)
        .await?;

    Ok(match accept_type {
        AcceptType::HTMX => templates::LeaderboardEntriesTemplate { entries }.into_response(),
        AcceptType::JSON => Json(entries).into_response(),
    })
}

pub async fn get_user_game_entries(
    accept_type: AcceptType,
    State(state): State<AppState>,
    Path((user_id, game_id)): Path<(Uuid, i32)>,
) -> Result<impl IntoResponse, ApiError> {
    let entries = sqlx::query_as::<_, LeaderboardEntry>(
        "SELECT * \
            FROM leaderboard_entries \
            WHERE game_id = $1 \
              AND user_id = $2\
            LIMIT 10;")
        .bind(game_id).bind(user_id)
        .fetch_all(&state.db)
        .await?;

    Ok(match accept_type {
        AcceptType::HTMX => templates::LeaderboardEntriesTemplate { entries }.into_response(),
        AcceptType::JSON => Json(entries).into_response(),
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

pub async fn create_game_entry(
    accept_type: AcceptType,
    State(state): State<AppState>,
    Path(game_id): Path<i32>,
    Extension(tx): Extension<LeaderboardStream>,
    JsonOrForm(request): JsonOrForm<LeaderboardEntryNew>,
) -> Result<impl IntoResponse, ApiError> {
    let leaderboard_entry = sqlx::query_as::<_, LeaderboardEntry>(
        "INSERT INTO leaderboard_entries (game_id, score, user_name, free_data, user_id) \
        VALUES ($1, $2, $3, $4, $5) \
        RETURNING id, score, game_id, user_name, free_data, user_id",
    );
    let leaderboard_entry = bind_all!(
        leaderboard_entry,
        game_id,
        request.score,
        request.user_name,
        request.free_data.unwrap_or("".into()),
        request.user_id.unwrap_or(Uuid::new_v4())
    );
    let leaderboard_entry = leaderboard_entry.fetch_one(&state.db).await?;

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
        AcceptType::HTMX => templates::LeaderboardEntryNewTemplate {
            entry: leaderboard_entry,
        }
        .into_response(),
        AcceptType::JSON => Json(leaderboard_entry).into_response(),
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
                    AcceptType::JSON => json.to_string(),
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
