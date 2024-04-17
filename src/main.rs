use sqlx::PgPool;
use log::info;


use fraculation_leaderboard::*;

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] db: PgPool) -> shuttle_axum::ShuttleAxum {
    info!("Running database migration");
    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Looks like something went wrong with migrations :(");

    let router = router::init_router(db);

    Ok(router.into())
}
