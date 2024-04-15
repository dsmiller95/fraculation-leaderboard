use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[derive(PartialEq, Clone)]
pub enum MutationKind {
    Create,
    Delete,
}
