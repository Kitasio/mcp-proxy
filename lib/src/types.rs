use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddParams {
    pub a: i64,
    pub b: i64,
}
