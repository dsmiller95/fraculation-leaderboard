mod common;

use common::my_test_server::*;
use common::test_models::*;
use fraculation_leaderboard::leaderboard::models::Game;

use assert_json_diff::assert_json_include;
use serde::Deserialize;
use serde_json::json;

mod game {
    use super::*;

    mod create {
        use super::*;

        #[tokio::test]
        async fn creates_game() {
            let server = get_app().await;
            get_app().await;

            let req = json!({
                "description": "Test Game Description 3123123",
                "score_sort_mode": "LesserIsBetter",
            });

            let response = server
                .post_json("/leaderboard/games", &req)
                .await
                .json::<serde_json::Value>();

            assert_json_include!(actual: response, expected: json!({
                "description": "Test Game Description 3123123",
                "score_sort_mode": "LesserIsBetter",
            }));
        }

        #[tokio::test]
        async fn default_to_higher_is_better() {
            let server = get_app().await;

            let req = json!({
                "description": "Test Game Description 287989",
            });

            let response = server
                .post_json("/leaderboard/games", &req)
                .await
                .json::<serde_json::Value>();

            assert_json_include!(actual: response, expected: json!({
                "description": "Test Game Description 287989",
                "score_sort_mode": "HigherIsBetter",
            }));
        }
    }

    mod get_all {
        use super::*;
        #[tokio::test]
        async fn gets_some_games() {
            let server = get_app().await;

            let response = server
                .get("/leaderboard/games")
                .await
                .json::<Vec<serde_json::Value>>();

            assert!(response.len() > 1);
        }
    }

    mod get_one {
        use super::*;
        #[tokio::test]
        async fn returns_same_data_as_create() {
            let server = get_app().await;

            let req = json!({
                "description": "Test Game Description 44523525",
            });

            let create_resp = server
                .post_json("/leaderboard/games", &req)
                .await
                .json::<Game>();
            let game_id = create_resp.id;

            let get_resp = server
                .get(format!("/leaderboard/games/{}", game_id).as_str())
                .await
                .json::<serde_json::Value>();

            assert_json_include!(actual: get_resp, expected: create_resp);
        }
    }

    mod entries {
        use super::*;

        mod create {
            use super::*;

            #[tokio::test]
            async fn creates_under_game() {
                let server = get_app().await;

                let req = json!({
                    "description": "Test Game Description 55111",
                });

                let new_game = server
                    .post_json("/leaderboard/games", &req)
                    .await
                    .json::<HasId>();

                let req = json!({
                    "score": 12.0,
                    "user_name": "bause",
                });
                let new_entry = server
                    .post_json(
                        format!("/leaderboard/games/{}/entries", new_game.id).as_str(),
                        &req,
                    )
                    .await
                    .json::<serde_json::Value>();

                assert_json_include!(actual: new_entry, expected: json!({
                    "game_id": new_game.id,
                    "score": 12.0,
                    "user_name": "bause",
                    "free_data": "",
                }));
            }
        }

        mod get_all {
            use super::*;

            async fn create_then_get_entries(
                server: impl MyTestServer,
                game_id: i32,
                scores: Vec<f64>,
                expected_scores: Vec<f64>,
            ) {
                #[derive(Deserialize)]
                struct HasGameId {
                    pub game_id: i32,
                }
                #[derive(Deserialize)]
                struct HasScore {
                    pub score: f64,
                }
                for score in scores {
                    let req = json!({
                        "score": score,
                        "user_name": "XXXX",
                    });
                    let new_entry = server
                        .post_json(
                            format!("/leaderboard/games/{}/entries", game_id).as_str(),
                            &req,
                        )
                        .await
                        .json::<HasGameId>();
                    assert_eq!(game_id, new_entry.game_id);
                }
                let all_entries = server
                    .get(format!("/leaderboard/games/{}/entries", game_id).as_str())
                    .await
                    .json::<Vec<HasScore>>();

                let actual_scores = all_entries.iter().map(|x| x.score).collect::<Vec<_>>();
                assert_eq!(expected_scores, actual_scores);
            }

            #[tokio::test]
            async fn sorts_by_score() {
                let server = get_app().await;

                let req = json!({
                    "description": "Test Game Description 8737312",
                    "score_sort_mode": "HigherIsBetter"
                });

                let new_game = server
                    .post_json("/leaderboard/games", &req)
                    .await
                    .json::<HasId>();

                create_then_get_entries(server, new_game.id, vec![13.0, 22.0], vec![22.0, 13.0])
                    .await
            }
            #[tokio::test]
            async fn sorts_by_inverse_score() {
                let server = get_app().await;

                let req = json!({
                    "description": "Test Game Description 8737312",
                    "score_sort_mode": "LesserIsBetter"
                });

                let new_game = server
                    .post_json("/leaderboard/games", &req)
                    .await
                    .json::<HasId>();

                create_then_get_entries(
                    server,
                    new_game.id,
                    vec![13.5, 22.0, 13.0],
                    vec![13.0, 13.5, 22.0],
                )
                .await
            }
        }
    }
}
