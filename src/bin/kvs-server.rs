use clap::{arg, value_parser, Command, Arg};
use kvs::KvStore;
use kvs::Result;
use slog::Drain;
use slog_scope::GlobalLoggerGuard;
use std::fs::{OpenOptions, File};
use std::io::Stderr;
use std::net::SocketAddr;
#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_scope;
#[macro_use]
extern crate log;

const SLOG_FILE_NAME: &str = "slog.log";

fn init_slog() -> GlobalLoggerGuard {
    let work_dir = std::env::current_dir().unwrap();
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(work_dir.join(SLOG_FILE_NAME)).unwrap();

    let decorator = slog_term::PlainSyncDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let logger = slog::Logger::root(drain, o!());
    
    // slog_stdlog uses the logger from slog_scope, so set a logger there
    let guard = slog_scope::set_global_logger(logger);

    // register slog_stdlog as the log handler with the log crate
    slog_stdlog::init().unwrap();

    guard
}

fn main() -> Result<()> {
    let _guard = init_slog();

    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let matches = Command::new(name)
        .version(version)
        .arg(Arg::new("ADDRESS").long("addr").value_parser(value_parser!(SocketAddr)).required(false))
        .arg(Arg::new("ENGINE_NAME").long("engine").value_parser(value_parser!(String)).required(true))
        .arg(arg!([REDUNDANT]).value_parser(value_parser!(String)))
        .get_matches();
    let mut kvs = KvStore::new()?;
    let mut listening_ip_port: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    let mut engine_name = String::new();

    let redundant = matches.get_one::<String>("REDUNDANT");
    assert!(redundant.is_none(), "has redundant argument!");
    if let Some(cmd_ip_port) = matches.get_one::<SocketAddr>("ADDRESS") {
        listening_ip_port = *cmd_ip_port;
    }
    if let Some(eng_name) = matches.get_one::<String>("ENGINE_NAME") {
        if eng_name.as_str() == "kvs" {

        } else if eng_name.as_str() == "sled" {

        } else {
            panic!("eng name should be either kvs or sled!");
        }
        engine_name = String::from(eng_name);
    }

    info!("package name: {}, version: {}", name, version);
    info!("[configuration] ip: {}, storage engine: {}", listening_ip_port, engine_name);

    eprintln!("package name: {}, version: {}", name, version);
    eprintln!("[configuration] ip: {}, storage engine: {}", listening_ip_port, engine_name);

    Ok(())
}
