use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Command {
  Set(Set),
  Rm(Remove),
}

#[derive(Serialize, Deserialize)]
pub struct Set {
  pub(crate) key: String,
  pub(crate) value: String,
}

#[derive(Serialize, Deserialize)]
pub struct Remove {
  pub(crate) key: String,
}
