use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone,
    utoipa::ToSchema)]
pub enum MutationKind {
    Create,
    Delete,
}
