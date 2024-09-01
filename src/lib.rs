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
pub mod thread_pool;

pub use engines::{KvStore, KvsEngine, SledStore};
pub use error::Result;
pub use logger::init_slog;
pub use cli::{KvsServer, KvsClient};
// pub use thread_pool::{ThreadPool, NaiveThreadPool};

