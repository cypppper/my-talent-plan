#[macro_use]
extern crate failure;

mod error;
mod wal;
mod engines;
mod cli;

pub use engines::{KvStore, KvsEngine};
pub use error::Result;
