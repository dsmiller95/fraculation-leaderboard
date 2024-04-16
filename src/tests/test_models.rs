use serde::Deserialize;

#[derive(Deserialize)]
pub struct HasId {
    pub id: i32,
}
