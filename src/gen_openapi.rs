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
    let doc = gen_my_openapi().to_pretty_json().unwrap();
    fs::write("./doc/openapi2.json", doc).expect("Should write to local file");
}