use serde::{Deserialize, Serialize};
use crate::models::MutationKind;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Game {
    pub id: i32,
    pub description: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct GameNew {
    pub description: String
}


#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub id: i32,
    pub score: f64,
    pub game_id: i32,
    pub user_name: String,
    pub free_data: String
}


#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct LeaderboardEntryNew {
    pub score: f32,
    pub user_name: String,
    pub free_data: Option<String>
}

#[derive(Clone, Serialize, Debug)]
pub struct LeaderboardUpdate{
    pub mutation_kind: MutationKind,
    pub id: i32
}
