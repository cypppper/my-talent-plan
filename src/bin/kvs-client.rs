use clap::Arg;
use clap::{arg, value_parser, Command};
use kvs::{KvStore, KvsClient, KvsEngine, Result};
use std::net::SocketAddr;
use std::process::exit;

fn main() -> Result<()> {
    let name = env!("CARGO_PKG_NAME");
    let authors = env!("CARGO_PKG_AUTHORS");
    let version = env!("CARGO_PKG_VERSION");
    let description = env!("CARGO_PKG_DESCRIPTION");
    let matches = Command::new(name)
        .version(version)
        .author(authors)
        .about(description)
        .subcommands([
            Command::new("set")
                .arg(arg!(<KEY>).value_parser(value_parser!(String)))
                .arg(arg!(<VALUE>).value_parser(value_parser!(String)))
                .arg(Arg::new("ADDRESS").long("addr").value_parser(value_parser!(SocketAddr)).required(false))
                .arg(arg!([REDUNDANT]).value_parser(value_parser!(String))),
            Command::new("get")
                .arg(arg!(<KEY>).value_parser(value_parser!(String)))
                .arg(Arg::new("ADDRESS").long("addr").value_parser(value_parser!(SocketAddr)).required(false))
                .arg(arg!([REDUNDANT]).value_parser(value_parser!(String))),
            Command::new("rm")
                .arg(arg!(<KEY>).value_parser(value_parser!(String)))
                .arg(Arg::new("ADDRESS").long("addr").value_parser(value_parser!(SocketAddr)).required(false))
                .arg(arg!([REDUNDANT]).value_parser(value_parser!(String))),
        ])
        .get_matches();
    let mut server_ip_port: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    match matches.subcommand() {
        Some(("set", set_cmd)) => {
            let key = set_cmd.get_one::<String>("KEY").unwrap();
            let value = set_cmd.get_one::<String>("VALUE").unwrap();
            if let Some(cmd_ip_port) = set_cmd.get_one::<SocketAddr>("ADDRESS") {
                server_ip_port = cmd_ip_port.to_owned();
            }
            let redundant = set_cmd.get_one::<String>("REDUNDANT");
            assert!(redundant.is_none(), "has redundant argument!");

            let mut kvclient = KvsClient::connect(server_ip_port).unwrap();
            kvclient.set(key, value, server_ip_port);


            // println!("set was used with key:{:?}, value:{:?}", key, value,);
            // if let Err(err) = kvs.set(key.to_owned(), value.to_owned()) {
            //     println!("{:?}", err);
            //     exit(-1);
            // } else {
            //     exit(0);
            // }
        }
        Some(("get", get_cmd)) => {
            let key = get_cmd.get_one::<String>("KEY").unwrap();
            if let Some(cmd_ip_port) = get_cmd.get_one::<SocketAddr>("ADDRESS") {
                server_ip_port = cmd_ip_port.to_owned();
            }
            let redundant = get_cmd.get_one::<String>("REDUNDANT");
            assert!(redundant.is_none(), "has redundant argument!");
  
            let mut kvclient = KvsClient::connect(server_ip_port).unwrap();
            kvclient.get(key, server_ip_port);

            // println!("get was used with key:{:?}", key,);
            // if let Ok(Some(value)) = kvs.get(key.to_owned()) {
            //     println!("{}", value);
            //     exit(0);
            // } else {
            //     println!("Key not found");
            //     exit(0);
            // }
        }
        Some(("rm", rm_cmd)) => {
            let key = rm_cmd.get_one::<String>("KEY").unwrap();
            if let Some(cmd_ip_port) = rm_cmd.get_one::<SocketAddr>("ADDRESS") {
                server_ip_port = cmd_ip_port.to_owned();
            }
            let redundant = rm_cmd.get_one::<String>("REDUNDANT");
            assert!(redundant.is_none(), "has redundant argument!");
           
            let mut kvclient = KvsClient::connect(server_ip_port).unwrap();
            kvclient.remove(key, server_ip_port);
           
            // println!("rm was used with key:{:?}", key,);
            // if let Ok(_) = kvs.remove(key.to_owned()) {
            //     exit(0);
            // } else {
            //     println!("Key not found");
            //     exit(-1);
            // }
        }
        _ => {
            unreachable!();
        }
    }
    Ok(())
}
