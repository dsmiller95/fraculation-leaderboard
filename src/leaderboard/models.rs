use serde::{Deserialize, Serialize};
use crate::models::MutationKind;

#[derive(Serialize, Deserialize, Debug)]
#[derive(PartialEq)]
#[derive(sqlx::Type)]
#[sqlx(rename_all = "PascalCase")]
#[sqlx(type_name = "GameScoreSortMode")]
pub enum GameScoreSortMode {
    HigherIsBetter,
    LesserIsBetter
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(PartialEq)]
#[derive(sqlx::FromRow)]
pub struct Game {
    pub id: i32,
    pub description: String,
    pub score_sort_mode: GameScoreSortMode
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(sqlx::FromRow)]
pub struct GameNew {
    pub description: String,
    pub score_sort_mode: Option<GameScoreSortMode>
}


#[derive(Serialize, Deserialize, Debug)]
#[derive(sqlx::FromRow)]
pub struct LeaderboardEntry {
    pub id: i32,
    pub score: f64,
    pub game_id: i32,
    pub user_name: String,
    pub free_data: String
}


#[derive(Serialize, Deserialize, Debug)]
#[derive(sqlx::FromRow)]
pub struct LeaderboardEntryNew {
    pub score: f64,
    pub user_name: String,
    pub free_data: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(Clone)]
pub struct LeaderboardUpdate{
    pub mutation_kind: MutationKind,
    pub id: i32
}
