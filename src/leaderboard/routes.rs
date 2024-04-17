use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    response::{sse::Event, IntoResponse, Sse},
    Extension, Json,
};
use axum::http::StatusCode;
use log::error;
use serde_json::json;
use sqlx::PgPool;
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

const NOT_FOUND_RESP: (StatusCode, &str) = (StatusCode::NOT_FOUND, "Not Found");

/// Get the home page
///
/// Responds with a htmx template
pub async fn home() -> impl IntoResponse {
    templates::HelloTemplate
}

pub async fn stream() -> impl IntoResponse {
    templates::StreamTemplate
}

/// Get the game list
///
/// Responds with a list of games
#[utoipa::path(
    get,
    path = "/leaderboard/games",
    responses(
        (status = 200, description = "Games list", body = Vec<Game>)
    )
)]
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

/// Create a game
///
/// Responds with the created game
#[utoipa::path(
    post,
    path = "/leaderboard/games",
    request_body = GameNew,
    responses(
        (status = 200, description = "New Game", body = Game)
    )
)]
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

async fn get_game_internal(db: &PgPool, game_id: i32) -> Result<Option<Game>, ApiError> {
    let entry = sqlx::query_as::<_, Game>(
        "SELECT * FROM games WHERE id = $1;")
        .bind(game_id)
        .fetch_optional(db)
        .await?;

    Ok(entry)
}

/// Get a game by id
///
/// Responds with full details about the game
#[utoipa::path(
    get,
    path = "/leaderboard/games/{id}",
    responses(
        (status = 200, description = "Game", body = Game),
        (status = 404, description = "Game not found", body = String, example = json!("Not Found")),
    )
)]
pub async fn get_game(
    accept_type: AcceptType,
    State(state): State<AppState>,
    Path(game_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let Some(game) = get_game_internal(&state.db, game_id).await? else {
        return Ok(NOT_FOUND_RESP.into_response());
    };

    Ok(match accept_type {
        AcceptType::HTMX => templates::GameTemplate { game }.into_response(),
        AcceptType::JSON => Json(game).into_response(),
    })
}

/// Get the entries for a game
///
/// Responds with a list of game entries, sorted by score based on the score_sort_mode of the game
#[utoipa::path(
    get,
    path = "/leaderboard/games/{id}/entries",
    responses(
        (status = 200, description = "Game Entries list", body = Vec<LeaderboardEntry>),
        (status = 404, description = "Game not found", body = String, example = json!("Not Found"))
    )
)]
pub async fn get_game_entries(
    accept_type: AcceptType,
    State(state): State<AppState>,
    Path(game_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let Some(game) =  get_game_internal(&state.db, game_id).await? else {
        return Ok(NOT_FOUND_RESP.into_response());
    };

    let ordering = match game.score_sort_mode {
        GameScoreSortMode::HigherIsBetter => "DESC",
        GameScoreSortMode::LesserIsBetter  => "ASC",
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

async fn get_user_game_entry_internal(game_id: i32, user_id: Uuid, db: &PgPool) -> Result<Option<LeaderboardEntry>, ApiError> {
    let entry = sqlx::query_as::<_, LeaderboardEntry>(
        "SELECT * \
            FROM leaderboard_entries \
            WHERE game_id = $1 \
              AND user_id = $2\
            LIMIT 1;")
        .bind(game_id).bind(user_id)
        .fetch_optional(db)
        .await?;

    Ok(entry)
}

/// Get an entry for a user on a specific game
///
/// Responds with a single game entry
#[utoipa::path(
    get,
    path = "/leaderboard/users/{user_id}/games/{game_id}/entries",
    responses(
        (status = 200, description = "Game Entry", body = LeaderboardEntry),
        (status = 404, description = "Game or Entry not found", body = String, example = json!("Not Found")),
    )
)]
pub async fn get_user_game_entry(
    accept_type: AcceptType,
    State(state): State<AppState>,
    Path((user_id, game_id)): Path<(Uuid, i32)>,
) -> Result<impl IntoResponse, ApiError> {
    let entry = get_user_game_entry_internal(game_id, user_id, &state.db).await?;

    let Some(entry) = entry else {
        return Ok(NOT_FOUND_RESP.into_response());
    };

    Ok(match accept_type {
        AcceptType::HTMX => templates::LeaderboardEntriesTemplate { entries: vec![entry] }.into_response(),
        AcceptType::JSON => Json(entry).into_response(),
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

async fn try_get_better_or_equal_entry(db: &PgPool, game_id: i32, user_id: Uuid, better_than: f64) -> Result<Option<LeaderboardEntry>, ApiError> {
    let existing_entry = get_user_game_entry_internal(game_id, user_id, db).await?;
    let Some(existing_entry) = existing_entry else { return Ok(None); };
    let Some(game) = get_game_internal(db, game_id).await? else { panic!("Game not found") };

    let existing_score = existing_entry.score;
    let is_existing_better_or_equal = match game.score_sort_mode{
        GameScoreSortMode::HigherIsBetter => existing_score >= better_than,
        GameScoreSortMode::LesserIsBetter => existing_score <= better_than,
    };
    Ok(match is_existing_better_or_equal {
        true => Some(existing_entry),
        false => None,
    })
}

/// Create a new game entry for a user
///
/// Responds with the created entry, or an existing entry, if new entry is not an improvement.
#[utoipa::path(
    post,
    path = "/leaderboard/games",
    request_body = LeaderboardEntryNew,
    responses(
        (status = 200, description = "New game entry", body = LeaderboardEntry),
        (status = 409, description = "Old, better, game entry", body = LeaderboardEntry)
    )
)]
pub async fn create_game_entry(
    accept_type: AcceptType,
    State(state): State<AppState>,
    Path(game_id): Path<i32>,
    Extension(tx): Extension<LeaderboardStream>,
    JsonOrForm(request): JsonOrForm<LeaderboardEntryNew>,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(ex_user_id) = request.user_id {
        if let Some(better_entry) = try_get_better_or_equal_entry(&state.db, game_id, ex_user_id, request.score).await? {
            return Ok((StatusCode::CONFLICT, Json(better_entry)).into_response())
        }
    }

    let leaderboard_entry = sqlx::query_as::<_, LeaderboardEntry>(
        "INSERT INTO leaderboard_entries (game_id, score, user_name, free_data, user_id) \
        VALUES ($1, $2, $3, $4, $5) \
        ON CONFLICT (game_id, user_id) DO UPDATE SET \
        score = EXCLUDED.score, user_name = EXCLUDED.user_name, free_data = EXCLUDED.free_data \
        RETURNING id, score, game_id, user_name, free_data, user_id \
        ",
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
        error!(
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
