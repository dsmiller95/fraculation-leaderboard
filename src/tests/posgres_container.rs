use std::future::Future;
use sqlx::PgPool;
use testcontainers::clients::Cli;
use testcontainers_modules::postgres::Postgres;
use once_cell::sync::Lazy;

pub static TEST_CONTAINER: Lazy<PgPool> = Lazy::new(|| {
    setup_sync_ish()
});


fn setup_sync_ish() -> impl Future<Output = PgPool> {
    setup()
}

async fn setup() -> PgPool {
    let docker = Cli::default();
    let node = docker.run(Postgres::default());

    // prepare connection string
    let connection_string = &format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        node.get_host_port_ipv4(5432)
    );

    let db: PgPool = PgPool::connect(&connection_string).await.unwrap();

    db
}