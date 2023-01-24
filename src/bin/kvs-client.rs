#![feature(let_chains)]
#![feature(is_some_and)]

use std::io::{Read, Write};
use std::net::TcpStream;
use clap::{Parser, Subcommand};
use kvs::engine::{Result};
use std::string::String;
use serde_resp::{array, bulk, RESPType};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "IP-PORT")]
    addr: Option<String>,
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Demo program that demonstrates the usage of \"KvStore\" core", long_about = None)]
    Get {
        key: String
    },
    #[command(about = "Set key-value string pair into kv store", long_about = None)]
    Set {
        key: String,
        value: String
    },
    #[command(about = "Remove key-value string from kv store with given key", long_about = None)]
    #[command(name="rm")]
    Remove {
        key: String
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let addr = if let Some(addr) = cli.addr { addr } else { "127.0.0.1:4000".to_owned() };
    let mut stream = TcpStream::connect(addr)?;
    match &cli.command {
        Commands::Set { key, value } => {
            // let command = RESPType::Array(vec![
            //     RESPType::BulkString("set".as_bytes().to_vec()),
            //     RESPType::BulkString(key.as_bytes().to_vec()),
            //     RESPType::BulkString(value.as_bytes().to_vec())
            // ]);
            let command = array!(bulk!("set"), bulk!(key), bulk!(value));
            let cmd_str = serde_resp::ser::to_string(&command).unwrap();
            stream.write(cmd_str.as_bytes())?;
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            println!("{response}");
            Ok(())
        },
        Commands::Get { key } => {
            // let command = RESPType::Array(vec![
            //     RESPType::BulkString("get".as_bytes().to_vec()),
            //     RESPType::BulkString(key.as_bytes().to_vec()),
            // ]);
            let command = array!(bulk!("get"), bulk!(key));
            let cmd_str= serde_resp::ser::to_string(&command).unwrap();
            stream.write(cmd_str.as_bytes())?;
            stream.flush()?;
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            let resp: RESPType = serde_resp::de::from_str(&response).unwrap();
            // currently only bulk str
            match resp {
                RESPType::BulkString(buf) => {
                    println!("{}", String::from_utf8(buf).unwrap())
                }
                _ => { println!() }
            }
            Ok(())
        },
        Commands::Remove { key } => {
            // let command = RESPType::Array(vec![
            //     RESPType::BulkString("rm".as_bytes().to_vec()),
            //     RESPType::BulkString(key.as_bytes().to_vec())
            // ]);
            let command = array!(bulk!("rm"), bulk!(key));
            let cmd_str = serde_resp::to_string(&command).unwrap();
            stream.write(cmd_str.as_bytes())?;
            stream.flush()?;
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            let resp: RESPType = serde_resp::from_str(&response).unwrap();
            match resp {
                RESPType::SimpleString(str) => {
                    println!("{str}");
                },
                RESPType::None => {
                    println!("Key not found");
                }
                _ => {}
            }
            Ok(())
        }
    }
    // match &cli.command {
    //     Commands::Set { key, value } => {
    //         let mut store = KvStore::open(".")?;
    //         store.set(key, value)
    //     },
    //     Commands::Get { key } => {
    //         let mut store = KvStore::open(".")?;
    //         if let Some(value) = store.get(key)? {
    //             println!("{value}");
    //             Ok(())
    //         } else {
    //             println!("Key not found: {key}");
    //         }
    //     },
    //     Commands::Remove { key } => {
    //         let mut store = KvStore::open(".")?;
    //         if store.remove(key)? == None {
    //             println!("Key not found");
    //             Err(KvError::KeyNotFound(key.to_owned()))
    //         } else {
    //             Ok(())
    //         }
    //     }
    // }
}
