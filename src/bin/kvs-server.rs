use std::fmt::{Display, Formatter};
use clap::{Parser, ValueEnum};
use kvs::engine::Result;

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
    let mut args = Args::parse();
    if args.engine == None {
        args.engine = Some(Engine::Kvs);
    }
    if args.addr == None {
        args.addr = Some("127.0.0.1:4000".to_owned())
    }
    log::info!("Listening to {}", args.addr.unwrap());
    log::info!("Using engine \"{}\"", args.engine.unwrap());

    Ok(())
}