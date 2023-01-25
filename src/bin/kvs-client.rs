#![feature(let_chains)]
#![feature(is_some_and)]

use std::io::{Read, Write};
use std::net::TcpStream;
use clap::{Parser, Subcommand};
use kvs::{KvsClient, Request, Result};
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
    let mut client = KvsClient::connect(addr)?;
    match &cli.command {
        Commands::Set { key, value } => client.set(key, value)?,
        Commands::Get { key } => {
            match client.get(key)? {
                Some(value) => println!("{value}"),
                None => println!()
            }
        },
        Commands::Remove { key } => {
            match client.rm(key)? {
                Some(msg) => println!("{msg}"),
                None => println!("Key not found")
            }
        }
    }
    Ok(())
}
