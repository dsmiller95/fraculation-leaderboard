use crate::tests::my_test_server::get_app;
use crate::tests::test_models::*;
use assert_json_diff::{assert_json_eq};
use axum::http::StatusCode;
use serde_json::json;
use sqlx::types::Uuid;

use super::*;

fn get_unique_user_id() -> Uuid {
    Uuid::new_v4()
}
#[tokio::test]
async fn two_scores_with_same_user_overwrites_when_higher() {
    let server = get_app().await;

    let req = json!({
        "description": "Test Game Description 7729653445",
    });

    let new_game = server
        .post_json("/leaderboard/games", &req)
        .await
        .json::<HasId>();
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
    let req = json!({
        "description": "Test Game Description df98h23",
    });
    let new_game = server
        .post_json("/leaderboard/games", &req)
        .await
        .json::<HasId>();
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

    let req = json!({
        "description": "Test Game Description 88241",
    });

    let new_game = server
        .post_json("/leaderboard/games", &req)
        .await
        .json::<HasId>();

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
    assert_json_eq!(all_entries, json!([added_first]));
}
