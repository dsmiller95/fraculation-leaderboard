use sqlx::PgPool;
mod errors;
mod hetero_req_resp;
mod leaderboard;
mod models;
mod router;
#[cfg(test)]
mod tests;
mod todo;

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] db: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Looks like something went wrong with migrations :(");

    let router = router::init_router(db);

    Ok(router.into())
}
