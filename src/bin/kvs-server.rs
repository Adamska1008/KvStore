use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use clap::{Parser, ValueEnum};
use serde_resp::RESPType;
use kvs::engine::{KvStore, Result};

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

fn unwrap_bulk_str(resp: &RESPType) -> String {
    if let RESPType::BulkString(bulk_str) = resp {
        String::from_utf8(bulk_str.clone()).unwrap()
    } else {
        panic!("not a resp bulk str")
    }
}

fn read_to_end(stream: &mut TcpStream) -> String {
    let mut received = vec![];
    let mut buffer = [0u8; 512];
    loop {
        let bytes_read = stream.read(&mut buffer).unwrap();
        received.extend_from_slice(&buffer[..bytes_read]);
        if bytes_read < 512 {
            break;
        }
    }
    String::from_utf8(received).unwrap()
}

fn handle_connection(mut stream: TcpStream, kvs: &mut KvStore) {
    let input=  read_to_end(&mut stream);
    let command: RESPType = serde_resp::de::from_str(&input).unwrap();
    let arr = if let RESPType::Array(arr) = command { arr } else { panic!("not a resp array") };
    let cmd = unwrap_bulk_str(&arr[0]);
    match cmd.as_str() {
        "get" => {
            let key = unwrap_bulk_str(&arr[1]);
            log::debug!("receive command: get {}", key);
            let resp_value = if let Some(value) = kvs.get(&key).unwrap() {
                RESPType::BulkString(value.as_bytes().to_vec())
            } else {
                RESPType::None
            };
            let response = serde_resp::to_string(&resp_value).unwrap();
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        },
        "set" => {
            let key = unwrap_bulk_str(&arr[1]);
            let value = unwrap_bulk_str(&arr[2]);
            log::debug!("receive command: set {} {}", key, value);
            kvs.set(&key, &value).unwrap();
            let ok = RESPType::SimpleString("OK".to_owned());
            let response = serde_resp::to_string(&ok).unwrap();
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        },
        "rm" => {
            let key = unwrap_bulk_str(&arr[1]);
            log::debug!("receive command: rm {}", key);
            let resp_value = if kvs.remove(&key).unwrap() == None {
                RESPType::None
            } else {
                RESPType::ok()
            };
            let response = serde_resp::to_string(&resp_value).unwrap();
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        _ => {}
    }
}

fn main() -> Result<()> {
    env_logger::init();
    log::info!("Running kvs server version {}", env!("CARGO_PKG_VERSION"));
    let args = Args::parse();
    let addr = if let Some(addr) = args.addr {addr} else {"127.0.0.1:4000".to_owned()};
    log::info!("Listening to {}", addr);
    // log::info!("Using engine \"{}\"", args.engine.unwrap());

    let mut kvs = KvStore::open(".")?;
    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, &mut kvs);
    }
    Ok(())
}