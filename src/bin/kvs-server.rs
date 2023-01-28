use std::env;
use std::fmt::{Display, Formatter};
use clap::{arg, Parser, ValueEnum};
use kvs::{KvsServer, Result};
use kvs::engine::Sled;
use kvs::KvStore;

const DEFAULT_PORT: u16 = 4000;
const DEFAULT_ENGINE: Engine = Engine::Kvs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "PORT", value_parser = clap::value_parser!(u16).range(1..))]
    port: Option<u16>,
    #[arg(short, long, value_enum)]
    engine: Option<Engine>
}

#[derive(Debug, Copy, Clone ,PartialOrd, PartialEq, Ord, Eq, ValueEnum)]
enum Engine {
    Kvs,
    Sled
}

impl Display for Engine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Engine::Kvs => write!(f, "kvs"),
            Engine::Sled => write!(f, "sled")
        }
    }
}

fn main() -> Result<()> {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    log::info!("Running kvs server version {}", env!("CARGO_PKG_VERSION"));
    let args = Args::parse();
    let port = args.port.unwrap_or_else(|| DEFAULT_PORT);
    let addr = format!("127.0.0.1:{}", port);
    let engine = args.engine.unwrap_or_else(|| DEFAULT_ENGINE);
    match engine {
        Engine::Kvs =>  {
            let mut server = KvsServer::new(KvStore::open(".")?)?;
            log::info!("Listening to {}", addr);
            server.run(&addr)?;
        },
        Engine::Sled => {
            let mut server = KvsServer::new(Sled::new(sled::open("my_db")?))?;
            log::info!("Listening to {}", addr);
            server.run(&addr)?;
        }
    }
    Ok(())
}