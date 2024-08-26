#[macro_use]
extern crate failure;
#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_scope;
#[macro_use]
extern crate log;

mod error;
mod wal;
mod engines;
mod cli;
mod logger;

pub use engines::{KvStore, KvsEngine, SledStore};
pub use error::Result;
pub use logger::init_slog;
pub use cli::{KvsServer, KvsClient};


