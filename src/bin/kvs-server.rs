use std::fmt::{Display, Formatter};
use clap::{Parser, ValueEnum};
use kvs::{KvsServer, Result};
use kvs::engine::Sled;
use kvs::KvStore;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "IP-PORT")]
    addr: Option<String>,
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
    env_logger::init();
    log::info!("Running kvs server version {}", env!("CARGO_PKG_VERSION"));
    let args = Args::parse();
    let addr = if let Some(addr) = args.addr { addr } else { "127.0.0.1:4000".to_owned() };
    let engine = if let Some(engine) = args.engine { engine } else { Engine::Kvs };
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