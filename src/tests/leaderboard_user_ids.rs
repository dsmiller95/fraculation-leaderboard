use crate::tests::my_test_server::get_app;
use crate::tests::test_models::*;
use assert_json_diff::{assert_json_eq, assert_json_include};
use rand::Rng;
use serde_json::json;

use super::*;
#[tokio::test]
async fn two_scores_with_same_user_overwrites() {
    let server = get_app().await;

    let req = json!({
        "description": "Test Game Description 7729653445",
    });

    let new_game = server
        .post_json("/leaderboard/games", &req)
        .await
        .json::<HasId>();
    let game_entry_url = format!("/leaderboard/games/{}/entries", new_game.id);

    let mut rng = rand::thread_rng();
    let user_id = rng.gen::<[i8; 4]>().map(|x| x.to_string()).join("_");
    let req = json!({
        "score": 12.0,
        "user_name": "bause",
        "user_id": user_id
    });
    let added_first = server
        .post_json(game_entry_url.as_str(), &req)
        .await
        .json::<serde_json::Value>();

    let all_entries = server.get(game_entry_url.as_str()).await.json::<serde_json::Value>();
    assert_json_eq!(all_entries, json!([
        added_first
    ]));


    let req = json!({
        "score": 44.0,
        "user_name": "diffuser_name",
        "user_id": user_id
    });
    let added_second = server
        .post_json(game_entry_url.as_str(), &req)
        .await
        .json::<serde_json::Value>();

    let all_entries = server.get(game_entry_url.as_str()).await.json::<serde_json::Value>();
    assert_json_eq!(all_entries, json!([
        added_second
    ]));
}
