use sqlx::PgPool;

async fn setup() -> PgPool {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/postgres")
        .await.unwrap();

    pool
}
