#![allow(dead_code)]

mod openapi;
mod models;
mod hetero_req_resp;
mod leaderboard;
mod errors;
mod app_state;

use std::fs;
use crate::openapi::gen_my_openapi;

fn main() {
    let doc = gen_my_openapi().to_yaml().unwrap();
    fs::write("./doc/openapi2.yml", doc).expect("Should write to local file");
}

#[test]
fn generated_docs_are_up_to_date() {
    let path = "doc/openapi2.yml";
    let current = fs::read_to_string(path)
        .expect("The current openapi file must exist");
    let newest = gen_my_openapi().to_yaml().unwrap();

    assert_eq!(
        newest, current,
        "
============================================================
NOTE: The generated `{path}` file is not up to date.
Please run `cargo run --bin gen_api` and commit the changes.
============================================================
"
    )
}