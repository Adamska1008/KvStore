use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use serde_resp::RESPType;
use crate::{GetResponse, KvStore, RemoveResponse, SetResponse, tools};
use crate::Result;

pub struct KvsServer {
    engine: KvStore
}

impl KvsServer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            engine: KvStore::open(".")?
        })
    }

    pub fn run<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(&addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(err) = self.serve(stream) {
                        log::error!("Error on serving client: {}", err)
                    }
                }
                Err(err) => log::error!("Connection failed: {}", err)
            }
        }
        Ok(())
    }

    pub fn serve(&mut self, mut stream: TcpStream) -> Result<()> {
        let input = tools::read_to_end(&mut stream);
        let command: RESPType = serde_resp::from_str(&input)?;
        let arr = if let RESPType::Array(arr) = command { arr } else { panic!("not a resp array") };
        let cmd = tools::unwrap_bulk_str(&arr[0]);
        match cmd.as_str() {
            "get" => {
                let key = tools::unwrap_bulk_str(&arr[1]);
                log::debug!("receive command: get {}", key);
                let rsp: RESPType = GetResponse::Ok(self.engine.get(&key)?).into();
                serde_resp::to_writer(&rsp, &mut stream)?;
            },
            "set" => {
                let key = tools::unwrap_bulk_str(&arr[1]);
                let value = tools::unwrap_bulk_str(&arr[2]);
                log::debug!("receive command: set {} {}", key, value);
                let rsp: RESPType = SetResponse::Ok(self.engine.set(&key, &value)?).into();
                serde_resp::to_writer(&rsp, &mut stream)?;
            },
            "rm" => {
                let key = tools::unwrap_bulk_str(&arr[1]);
                log::debug!("receive command: rm {}", key);
                let rsp: RESPType = RemoveResponse::Ok(self.engine.remove(&key)?).into();
                serde_resp::to_writer(&rsp, &mut stream)?;
            }
            _ => {}
        }
        Ok(())
    }
}