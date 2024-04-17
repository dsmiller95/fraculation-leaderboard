use crate::tests::my_test_server::get_app;
use crate::tests::test_models::*;
use assert_json_diff::{assert_json_eq, assert_json_include};
use axum::http::StatusCode;
use serde_json::json;
use sqlx::types::Uuid;

use super::*;

fn get_unique_user_id() -> Uuid {
    Uuid::new_v4()
}
async fn create_game(server: &impl MyTestServer, desc: &str) -> HasId {
    let req = json!({
        "description": desc,
    });

    let x = server
        .post_json("/leaderboard/games", &req)
        .await
        .json::<HasId>();
    x
}

#[tokio::test]
async fn two_scores_with_same_user_overwrites_when_higher() {
    let server = get_app().await;

    let new_game = create_game(&server, "Test Game Description 7729653445").await;

    let game_entry_url = format!("/leaderboard/games/{}/entries", new_game.id);

    let user_id = get_unique_user_id();
    let req = json!({ "score": 12.0, "user_name": "bause", "user_id": user_id });
    let added_first = server
        .post_json(game_entry_url.as_str(), &req)
        .await
        .json::<serde_json::Value>();

    let all_entries = server
        .get(game_entry_url.as_str())
        .await
        .json::<serde_json::Value>();
    assert_json_eq!(all_entries, json!([added_first]));

    let req = json!({ "score": 44.0, "user_name": "diffuser_name", "user_id": user_id });
    let added_second = server
        .post_json(game_entry_url.as_str(), &req)
        .await
        .json::<serde_json::Value>();

    let all_entries = server
        .get(game_entry_url.as_str())
        .await
        .json::<serde_json::Value>();
    assert_json_eq!(all_entries, json!([added_second]));
}

#[tokio::test]
async fn two_scores_with_same_user_overwrites_when_lower_if_lower_is_better() {
    let server = get_app().await;

    let req = json!({
        "description": "Test Game Description 6y27912",
        "score_sort_mode": "LesserIsBetter"
    });

    let new_game = server
        .post_json("/leaderboard/games", &req)
        .await
        .json::<HasId>();
    let game_entry_url = format!("/leaderboard/games/{}/entries", new_game.id);
    let user_id = get_unique_user_id();

    let req = json!({ "score": 92.0, "user_name": "bause", "user_id": user_id });
    let _added_first = server
        .post_json(game_entry_url.as_str(), &req)
        .await
        .json::<serde_json::Value>();

    let req = json!({ "score": 22.0, "user_name": "diffuser_name", "user_id": user_id });
    let added_second = server
        .post_json(game_entry_url.as_str(), &req)
        .await
        .json::<serde_json::Value>();

    let all_entries = server
        .get(game_entry_url.as_str())
        .await
        .json::<serde_json::Value>();
    assert_json_eq!(all_entries, json!([added_second]));
}

#[tokio::test]
async fn two_scores_with_same_user_conflicts_when_lower() {
    let server = get_app().await;
    let new_game = create_game(&server, "Test Game Description df98h23").await;

    let game_entry_url = format!("/leaderboard/games/{}/entries", new_game.id);
    let user_id = get_unique_user_id();

    let req = json!({ "score": 82.0, "user_name": "bause", "user_id": user_id });
    let added_first = server
        .post_json(game_entry_url.as_str(), &req)
        .await
        .json::<serde_json::Value>();

    let all_entries = server
        .get(game_entry_url.as_str())
        .await
        .json::<serde_json::Value>();
    assert_json_eq!(all_entries, json!([added_first]));

    let req = json!({ "score": 24.0, "user_name": "diffuser_name", "user_id": user_id });
    let add_resp = server.post_json(game_entry_url.as_str(), &req).await;

    assert_eq!(StatusCode::CONFLICT, add_resp.status_code());
    let add_resp_json = add_resp.json_allow_fail::<serde_json::Value>();
    assert_json_eq!(added_first, json!([add_resp_json]));

    let all_entries = server
        .get(game_entry_url.as_str())
        .await
        .json::<serde_json::Value>();
    assert_json_eq!(all_entries, json!([added_first]));
}



#[tokio::test]
async fn creates_score_with_user_gets_score_by_user() {
    let server = get_app().await;

    let new_game = create_game(&server, "Test Game Description 88241").await;

    let user_id = get_unique_user_id();
    let req = json!({ "score": 12.0, "user_name": "simnes", "user_id": user_id });
    let added_first = server
        .post_json(format!("/leaderboard/games/{}/entries", new_game.id).as_str(), &req)
        .await
        .json::<serde_json::Value>();

    let all_entries = server
        .get(format!("/leaderboard/users/{}/games/{}/entries", user_id, new_game.id).as_str())
        .await
        .json::<serde_json::Value>();
    assert_json_eq!(all_entries, json!(added_first));
}

async fn create_game_entry(server: &impl MyTestServer, game_id: i32, score: f64, user_id: Uuid) -> serde_json::Value {
    let req = json!({ "score": score, "user_name": "simnes", "user_id": user_id });
    let x = server
        .post_json(format!("/leaderboard/games/{}/entries", game_id).as_str(), &req)
        .await
        .json::<serde_json::Value>();
    x
}

async fn get_game_entry(server: &impl MyTestServer, game_id: i32, user_id: Uuid) -> serde_json::Value {
    server
        .get(format!("/leaderboard/users/{}/games/{}/entries", user_id, game_id).as_str())
        .await
        .json::<serde_json::Value>()
}

#[tokio::test]
async fn creates_score_with_user_on_two_games_gets_score_by_user_on_games() {
    let server = get_app().await;

    let new_game_one = create_game(&server, "Test Game Description 08123hnjd1").await;
    let new_game_two = create_game(&server, "Test Game Description 98sdfkjl13").await;

    let user_id = get_unique_user_id();
    let added_game_one = create_game_entry(&server, new_game_one.id, 15.0, user_id).await;
    assert_json_include!(actual: added_game_one, expected: json!({"score": 15.0}));
    let added_game_two = create_game_entry(&server, new_game_two.id, 33.0, user_id).await;
    assert_json_include!(actual: added_game_two, expected: json!({"score": 33.0}));

    let game_one_entry = get_game_entry(&server, new_game_one.id, user_id).await;
    let game_two_entry = get_game_entry(&server, new_game_two.id, user_id).await;

    assert_json_eq!(game_one_entry, added_game_one);
    assert_json_eq!(game_two_entry, added_game_two);
}
