use crate::router::init_router;
use crate::tests::postgres::get_shared_pool;
use axum::http::header::ACCEPT;
use axum::http::HeaderValue;
use axum_test::{TestResponse, TestServer};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::IntoFuture;

pub async fn get_app() -> impl MyTestServer {
    let pg = get_shared_pool().await;
    let app = init_router(pg);
    let mut server = TestServer::new(app).unwrap();
    server.add_header(ACCEPT, HeaderValue::from_static("application/json"));

    server
}

pub trait MyTestServer {
    fn post_json<T>(&self, path: &str, json: &T) -> impl IntoFuture<Output = impl MyTestResponse>
    where
        T: ?Sized + Serialize;

    fn get(&self, path: &str) -> impl IntoFuture<Output = impl MyTestResponse>;
}

pub trait MyTestRequest {}
pub trait MyTestResponse {
    fn json<T>(&self) -> T
    where
        T: DeserializeOwned;
}

impl MyTestServer for TestServer {
    fn post_json<T>(&self, path: &str, json: &T) -> impl IntoFuture<Output = impl MyTestResponse>
    where
        T: ?Sized + Serialize,
    {
        self.post(path).json(&json).expect_success()
    }

    fn get(&self, path: &str) -> impl IntoFuture<Output = impl MyTestResponse> {
        self.get(path).expect_success()
    }
}

impl MyTestResponse for TestResponse {
    fn json<T>(&self) -> T
    where
        T: DeserializeOwned,
    {
        self.json()
    }
}
