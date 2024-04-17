use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{sse::Event, IntoResponse, Sse},
    Extension, Form,
};
use serde_json::json;
use std::convert::Infallible;
use std::time::Duration;
use log::error;
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt as _};

use super::models::{MutationKind, Todo, TodoNew, TodoUpdate};
use super::templates;
use crate::{errors::ApiError, app_state::AppState};

pub type TodosStream = Sender<TodoUpdate>;

pub async fn home() -> impl IntoResponse {
    templates::HelloTemplate
}

pub async fn stream() -> impl IntoResponse {
    templates::StreamTemplate
}

pub async fn fetch_todos(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let todos = sqlx::query_as::<_, Todo>("SELECT * FROM TODOS")
        .fetch_all(&state.db)
        .await?;

    Ok(templates::Records { todos })
}

pub async fn create_todo(
    State(state): State<AppState>,
    Extension(tx): Extension<TodosStream>,
    Form(form): Form<TodoNew>,
) -> impl IntoResponse {
    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO TODOS (description) VALUES ($1) RETURNING id, description",
    )
    .bind(form.description)
    .fetch_one(&state.db)
    .await
    .unwrap();

    if tx
        .send(TodoUpdate {
            mutation_kind: MutationKind::Create,
            id: todo.id,
        })
        .is_err()
    {
        error!(
            "Record with ID {} was created but nobody's listening to the stream!",
            todo.id
        );
    }

    templates::TodoNewTemplate { todo }
}
pub async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Extension(tx): Extension<TodosStream>,
) -> Result<impl IntoResponse, ApiError> {
    sqlx::query("DELETE FROM TODOS WHERE ID = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if tx
        .send(TodoUpdate {
            mutation_kind: MutationKind::Delete,
            id,
        })
        .is_err()
    {
        error!(
            "Record with ID {} was deleted but nobody's listening to the stream!",
            id
        );
    }

    Ok(StatusCode::OK)
}

pub async fn handle_stream(
    Extension(tx): Extension<TodosStream>,
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
