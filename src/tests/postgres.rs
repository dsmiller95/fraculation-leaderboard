use sqlx::PgPool;

pub async fn get_shared_pool() -> PgPool {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:15797/fraculation-leaderboard")
        .await.unwrap();

    pool
}
