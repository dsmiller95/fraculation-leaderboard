use crate::models::MutationKind;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq,
    sqlx::Type,
    utoipa::ToSchema)]
#[sqlx(rename_all = "PascalCase")]
#[sqlx(type_name = "GameScoreSortMode")]
pub enum GameScoreSortMode {
    HigherIsBetter,
    LesserIsBetter,
}

#[derive(Serialize, Deserialize, Debug, PartialEq,
    sqlx::FromRow,
    utoipa::ToSchema)]
pub struct Game {
    pub id: i32,
    pub description: String,
    pub score_sort_mode: GameScoreSortMode,
}

#[derive(Serialize, Deserialize, Debug,
    sqlx::FromRow,
    utoipa::ToSchema)]
pub struct GameNew {
    pub description: String,
    pub score_sort_mode: Option<GameScoreSortMode>,
}

#[derive(Serialize, Deserialize, Debug,
    sqlx::FromRow,
    utoipa::ToSchema)]
pub struct LeaderboardEntry {
    pub id: i32,
    pub score: f64,
    pub game_id: i32,
    pub user_name: String,
    pub user_id: Uuid,
    pub free_data: String,
}

#[derive(Serialize, Deserialize, Debug,
    sqlx::FromRow,
    utoipa::ToSchema)]
pub struct LeaderboardEntryNew {
    pub score: f64,
    pub user_name: String,
    /// When not provided, will be assigned a random unique id
    pub user_id: Option<Uuid>,
    pub free_data: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone,
    utoipa::ToSchema)]
pub struct LeaderboardUpdate {
    pub mutation_kind: MutationKind,
    pub id: i32,
}
