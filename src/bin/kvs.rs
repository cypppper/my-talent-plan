use clap::{arg, value_parser, Command};
use kvs::KvStore;
use kvs::Result;
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
                .arg(arg!([REDUNDANT]).value_parser(value_parser!(String))),
            Command::new("get")
                .arg(arg!(<KEY>).value_parser(value_parser!(String)))
                .arg(arg!([REDUNDANT]).value_parser(value_parser!(String))),
            Command::new("rm")
                .arg(arg!(<KEY>).value_parser(value_parser!(String)))
                .arg(arg!([REDUNDANT]).value_parser(value_parser!(String))),
        ])
        .get_matches();
    let mut kvs = KvStore::new()?;
    match matches.subcommand() {
        Some(("set", set_cmd)) => {
            let key = set_cmd.get_one::<String>("KEY").unwrap();
            let value = set_cmd.get_one::<String>("VALUE").unwrap();
            let redundant = set_cmd.get_one::<String>("REDUNDANT");
            assert!(redundant.is_none());
            // println!("set was used with key:{:?}, value:{:?}", key, value,);
            if let Err(err) = kvs.set(key.to_owned(), value.to_owned()) {
                println!("{:?}", err);
                exit(-1);
            } else {
                exit(0);
            }
        }
        Some(("get", get_cmd)) => {
            let key = get_cmd.get_one::<String>("KEY").unwrap();
            let redundant = get_cmd.get_one::<String>("REDUNDANT");
            assert!(redundant.is_none());
            // println!("get was used with key:{:?}", key,);
            if let Ok(Some(value)) = kvs.get(key.to_owned()) {
                println!("{}", value);
                exit(0);
            } else {
                println!("Key not found");
                exit(0);
            }
        }
        Some(("rm", rm_cmd)) => {
            let key = rm_cmd.get_one::<String>("KEY").unwrap();
            let redundant = rm_cmd.get_one::<String>("REDUNDANT");
            assert!(redundant.is_none());
            // println!("rm was used with key:{:?}", key,);
            if let Ok(_) = kvs.remove(key.to_owned()) {
                exit(0);
            } else {
                println!("Key not found");
                exit(-1);
            }
        }
        _ => {
            unreachable!();
        }
    }
    Ok(())
}
