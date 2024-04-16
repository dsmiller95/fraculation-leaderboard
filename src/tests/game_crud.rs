use crate::leaderboard::models::{Game, GameNew, GameScoreSortMode};

use crate::tests::my_test_server::*;
mod game {
    use super::*;

    mod create {
        use super::*;

        #[tokio::test]
        async fn creates_game() {
            let server = get_app().await;

            let req = GameNew {
                description: "Test Game Description 3123123".into(),
                score_sort_mode: Some(GameScoreSortMode::LesserIsBetter),
            };

            let response = server
                .post_json("/leaderboard/games", &req)
                .await
                .json::<Game>();

            assert_eq!("Test Game Description 3123123", response.description);
            assert_eq!(GameScoreSortMode::LesserIsBetter, response.score_sort_mode);
        }

        #[tokio::test]
        async fn default_to_higher_is_better() {
            let server = get_app().await;

            let req = GameNew {
                description: "Test Game Description 287989".into(),
                score_sort_mode: None,
            };

            let response = server
                .post_json("/leaderboard/games", &req)
                .await
                .json::<Game>();

            assert_eq!("Test Game Description 287989", response.description);
            assert_eq!(GameScoreSortMode::HigherIsBetter, response.score_sort_mode);
        }
    }

    mod get_all {
        use super::*;
        #[tokio::test]
        async fn gets_some_games() {
            let server = get_app().await;

            let response = server.get("/leaderboard/games").await.json::<Vec<Game>>();

            assert!(response.len() > 1);
        }
    }

    mod get_one {
        use super::*;
        #[tokio::test]
        async fn returns_same_data_as_create() {
            let server = get_app().await;

            let req = GameNew {
                description: "Test Game Description 44523525".into(),
                score_sort_mode: None,
            };

            let create_resp = server
                .post_json("/leaderboard/games", &req)
                .await
                .json::<Game>();
            let game_id = create_resp.id;

            let get_resp = server
                .get(format!("/leaderboard/games/{}", game_id).as_str())
                .await
                .json::<Game>();

            assert_eq!(create_resp, get_resp);
        }
    }

    mod entries {
        use super::*;
        use crate::leaderboard::models::{LeaderboardEntry, LeaderboardEntryNew};

        mod create {
            use super::*;

            #[tokio::test]
            async fn creates_under_game() {
                let server = get_app().await;

                let req = GameNew {
                    description: "Test Game Description 55111".into(),
                    score_sort_mode: None,
                };

                let new_game = server
                    .post_json("/leaderboard/games", &req)
                    .await
                    .json::<Game>();

                let req = LeaderboardEntryNew {
                    score: 12.0,
                    user_name: "bause".to_string(),
                    free_data: None,
                };
                let new_entry = server
                    .post_json(
                        format!("/leaderboard/games/{}/entries", new_game.id).as_str(),
                        &req,
                    )
                    .await
                    .json::<LeaderboardEntry>();

                assert_eq!(new_game.id, new_entry.game_id);
                assert_eq!(12.0, new_entry.score);
                assert_eq!("bause", new_entry.user_name);
                assert_eq!("", new_entry.free_data);
            }
        }

        mod get_all {
            use super::*;
            use crate::leaderboard::models::GameScoreSortMode::{HigherIsBetter, LesserIsBetter};

            async fn create_then_get_entries(
                server: impl MyTestServer,
                game_id: i32,
                scores: Vec<f64>,
                expected_scores: Vec<f64>,
            ) {
                for score in scores {
                    let req = LeaderboardEntryNew {
                        score: score,
                        user_name: "XXXX".to_string(),
                        free_data: None,
                    };
                    let new_entry = server
                        .post_json(
                            format!("/leaderboard/games/{}/entries", game_id).as_str(),
                            &req,
                        )
                        .await
                        .json::<LeaderboardEntry>();
                    assert_eq!(game_id, new_entry.game_id);
                }

                let all_entries = server
                    .get(format!("/leaderboard/games/{}/entries", game_id).as_str())
                    .await
                    .json::<Vec<LeaderboardEntry>>();

                let actual_scores = all_entries.iter().map(|x| x.score).collect::<Vec<_>>();
                assert_eq!(expected_scores, actual_scores);
            }

            #[tokio::test]
            async fn sorts_by_score() {
                let server = get_app().await;

                let req = GameNew {
                    description: "Test Game Description 8737312".into(),
                    score_sort_mode: Some(HigherIsBetter),
                };

                let new_game = server
                    .post_json("/leaderboard/games", &req)
                    .await
                    .json::<Game>();

                create_then_get_entries(server, new_game.id, vec![13.0, 22.0], vec![22.0, 13.0])
                    .await
            }
            #[tokio::test]
            async fn sorts_by_inverse_score() {
                let server = get_app().await;

                let req = GameNew {
                    description: "Test Game Description 242424".into(),
                    score_sort_mode: Some(LesserIsBetter),
                };

                let new_game = server
                    .post_json("/leaderboard/games", &req)
                    .await
                    .json::<Game>();

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
