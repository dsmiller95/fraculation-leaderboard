use crate::leaderboard::models::{Game, GameNew, GameScoreSortMode};
use crate::router::init_router;
use crate::tests::postgres::get_shared_pool;
use axum::http::header::ACCEPT;
use axum::http::HeaderValue;
use axum_test::TestServer;

async fn get_app() -> TestServer {
    let pg = get_shared_pool().await;
    let app = init_router(pg);
    let mut server = TestServer::new(app).unwrap();
    server.add_header(ACCEPT, HeaderValue::from_static("application/json"));

    server
}

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
                .post("/leaderboard/games")
                .json(&req)
                .expect_success()
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
                .post("/leaderboard/games")
                .json(&req)
                .expect_success()
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
                .post("/leaderboard/games")
                .json(&req)
                .expect_success()
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
                    .post("/leaderboard/games")
                    .json(&req)
                    .expect_success()
                    .await
                    .json::<Game>();

                let req = LeaderboardEntryNew {
                    score: 12.0,
                    user_name: "bause".to_string(),
                    free_data: None,
                };
                let new_entry = server
                    .post(format!("/leaderboard/games/{}/entries", new_game.id).as_str())
                    .json(&req)
                    .expect_success()
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
                server: TestServer,
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
                        .post(format!("/leaderboard/games/{}/entries", game_id).as_str())
                        .json(&req)
                        .expect_success()
                        .await
                        .json::<LeaderboardEntry>();
                    assert_eq!(game_id, new_entry.game_id);
                }

                let all_entries = server
                    .get(format!("/leaderboard/games/{}/entries", game_id).as_str())
                    .expect_success()
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
                    .post("/leaderboard/games")
                    .json(&req)
                    .expect_success()
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
                    .post("/leaderboard/games")
                    .json(&req)
                    .expect_success()
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
