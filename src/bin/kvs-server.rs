use clap::{arg, value_parser, Command, Arg};
use kvs::{init_slog, thread_pool::{SharedQueueThreadPool, ThreadPool}, KvStore, KvsEngine, SledStore, KvsServer, Result};
use std::net::SocketAddr;
#[macro_use]
extern crate log;

fn main() -> Result<()> {
    let _guard = init_slog();

    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let matches = Command::new(name)
        .version(version)
        .arg(Arg::new("ADDRESS").long("addr").value_parser(value_parser!(SocketAddr)).required(false))
        .arg(Arg::new("ENGINE_NAME").long("engine").value_parser(value_parser!(String)).required(false))
        .arg(arg!([REDUNDANT]).value_parser(value_parser!(String)))
        .get_matches();
    let mut listening_ip_port: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    let mut engine_name = String::from("kvs");

    let redundant = matches.get_one::<String>("REDUNDANT");
    assert!(redundant.is_none(), "has redundant argument!");
    if let Some(cmd_ip_port) = matches.get_one::<SocketAddr>("ADDRESS") {
        listening_ip_port = *cmd_ip_port;
    }
    if let Some(eng_name) = matches.get_one::<String>("ENGINE_NAME") {
        engine_name = String::from(eng_name);
    }

    info!("package name: {}, version: {}", name, version);
    info!("[configuration] ip: {}, storage engine: {}", listening_ip_port, engine_name);

    eprintln!("package name: {}, version: {}", name, version);
    eprintln!("[configuration] ip: {}, storage engine: {}", listening_ip_port, engine_name);


    if engine_name.as_str() == "kvs" {
        let kvserver = KvsServer::new(
            listening_ip_port, 
            KvStore::open(std::env::current_dir().unwrap()).unwrap(),
            SharedQueueThreadPool::new(4).unwrap()
        );
        kvserver.start();
    } else {
        let kvserver = KvsServer::new(
            listening_ip_port, 
            SledStore::open(std::env::current_dir().unwrap()).unwrap(),
            SharedQueueThreadPool::new(4).unwrap()
        );
        kvserver.start();
    }


    Ok(())
}
