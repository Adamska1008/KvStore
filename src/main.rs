#![feature(let_chains)]
#![feature(is_some_and)]

use clap::{arg, Command};
use kvs::{KvError, Result};
use kvs::KvStore;
use std::string::String;

fn main() -> Result<()> {
    let matches = Command::new("kvs")
        .version("0.1.0")
        .author("Zijian Zang <2639980868@qq.com>")
        .about("Demo program that demonstrates the usage of \"KvStore\" core.")
        // .arg(arg!(-V --version "version info"))
        .subcommand(
            Command::new("get")
                .about("get string value from kv store with given key")
                .arg(arg!([key]).required(true))
        )
        .subcommand(
            Command::new("set")
                .about("set key-value string pair into kv store")
                .args([arg!([key]).required(true), arg!([value]).required(true)])
        )
        .subcommand(
            Command::new("rm")
                .about("remove key-value string from kv store with given key")
                .arg(arg!([key]).required(true))
        )
        .get_matches();

    match matches.subcommand() {
        Some(("set", sub_matches)) => {
            let mut store = KvStore::open(".")?;
            if let Some(key) = sub_matches.get_one::<String>("key") &&
                let Some(value) = sub_matches.get_one::<String>("value") {
                store.set(key, value)?;
                Ok(())
            } else {
                Err(KvError::MissingArguments)
            }
        },
        Some(("get", sub_matches)) => {
            let mut store = KvStore::open(".")?;
            if let Some(key) = sub_matches.get_one::<String>("key") {
                if let Some(value) = store.get(key)? {
                    println!("{}", value);
                    Ok(())
                } else {
                    println!("Key not found: {key}");
                    Err(KvError::KeyNotFound(key.to_string()))
                }
            } else {
                Err(KvError::MissingArguments)
            }
        },
        Some(("rm", sub_matches)) => {
            let mut store = KvStore::open(".")?;
            if let Some(key) = sub_matches.get_one::<String>("key") {
                if store.remove(key)? == None {
                    println!("Key not found");
                    Err(KvError::KeyNotFound(key.to_owned()))
                }  else {
                    Ok(())
                }
            } else {
                Err(KvError::MissingArguments)
            }
        },
        _ => {
            Err(KvError::UnknownCommand)
        }
    }
}
