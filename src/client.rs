use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};
use serde_resp::RESPType;
use crate::{Request, Result};

pub struct KvsClient {
    stream: TcpStream
}

impl KvsClient {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(Self {
            stream
        })
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let command: RESPType = Request::set(key, value).into();
        serde_resp::to_writer(&command, &mut self.stream).unwrap();
        let mut response = String::new();
        self.stream.read_to_string(&mut response).unwrap();
        println!("{response}");
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Result<Option<String>> {
        let command: RESPType = Request::get(key).into();
        serde_resp::to_writer(&command, &mut self.stream).unwrap();
        let mut response = String::new();
        self.stream.read_to_string(&mut response).unwrap();
        let resp: RESPType = serde_resp::from_str(&response).unwrap();
        // currently only bulk str
        match resp {
            RESPType::BulkString(buf) => {
                Ok(Some(String::from_utf8(buf).unwrap()))
            }
            _ => { Ok(None) }
        }
    }

    pub fn rm(&mut self, key: &str) -> Result<Option<String>> {
        let command: RESPType = Request::remove(key).into();
        serde_resp::to_writer(&command, &mut self.stream).unwrap();
        let mut response = String::new();
        self.stream.read_to_string(&mut response).unwrap();
        let resp: RESPType = serde_resp::from_str(&response).unwrap();
        match resp {
            RESPType::SimpleString(str) => {
                Ok(Some(str))
            },
            _ => Ok(None)
        }
    }
}