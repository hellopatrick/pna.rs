use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KVError {
  #[error("Key not found: {0}")]
  KeyNotFound(String),
  #[error("I/O Error: {0}")]
  Io(io::Error),
  #[error("Message Pack Error: {0}")]
  MessagePack(rmp_serde::encode::Error),
  #[error("Bincode Error: {0}")]
  Bincode(bincode::Error),
}

impl From<io::Error> for KVError {
  fn from(err: io::Error) -> KVError {
    KVError::Io(err)
  }
}

impl From<bincode::Error> for KVError {
  fn from(err: bincode::Error) -> KVError {
    KVError::Bincode(err)
  }
}

impl From<rmp_serde::encode::Error> for KVError {
  fn from(err: rmp_serde::encode::Error) -> KVError {
    KVError::MessagePack(err)
  }
}
