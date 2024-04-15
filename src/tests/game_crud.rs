use axum::http::header::ACCEPT;
use axum::http::HeaderValue;
use axum_test::TestServer;
use crate::leaderboard::models::{Game, GameNew, GameScoreSortMode};
use crate::router::init_router;
use crate::tests::postgres::get_shared_pool;

async fn get_app() -> TestServer {
    let pg = get_shared_pool().await;
    let app = init_router(pg);
    let mut server = TestServer::new(app).unwrap();
    server.add_header(ACCEPT, HeaderValue::from_static("application/json"));

    server
}


#[tokio::test]
async fn test_game_create() {
    let server = get_app().await;

    let req = GameNew{
        description: "Test Game Description 3123123".into(),
        score_sort_mode: Some(GameScoreSortMode::LesserIsBetter)
    };

    let response = server
        .post("/leaderboard/games").json(&req).expect_success()
        .await.json::<Game>();

    assert_eq!("Test Game Description 3123123", response.description);
    assert_eq!(GameScoreSortMode::LesserIsBetter, response.score_sort_mode);
}


#[tokio::test]
async fn test_game_create_default_to_higher_is_better() {
    let server = get_app().await;

    let req = GameNew{
        description: "Test Game Description 287989".into(),
        score_sort_mode: None
    };

    let response = server
        .post("/leaderboard/games").json(&req).expect_success()
        .await.json::<Game>();

    assert_eq!("Test Game Description 287989", response.description);
    assert_eq!(GameScoreSortMode::HigherIsBetter, response.score_sort_mode);
}