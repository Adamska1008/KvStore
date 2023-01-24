#![feature(let_chains)]
#![feature(is_some_and)]

use clap::{Parser, Subcommand};
use kvs::engine::{KvError, Result};
use kvs::engine::KvStore;
use std::string::String;

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
    let mut cli = Cli::parse();
    if cli.addr == None {
        cli.addr = Some("127.0.0.1:4000".to_owned());
    }
    match &cli.command {
        Commands::Set { key, value } => {
            let mut store = KvStore::open(".")?;
            store.set(key, value)
        },
        Commands::Get { key } => {
            let mut store = KvStore::open(".")?;
            if let Some(value) = store.get(key)? {
                println!("{value}");
                Ok(())
            } else {
                println!("Key not found: {key}");
                Err(KvError::KeyNotFound(key.to_owned()))
            }
        },
        Commands::Remove { key } => {
            let mut store = KvStore::open(".")?;
            if store.remove(key)? == None {
                println!("Key not found");
                Err(KvError::KeyNotFound(key.to_owned()))
            } else {
                Ok(())
            }
        }
    }
}
