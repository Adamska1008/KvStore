use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};
use serde_resp::{RESPType};
use crate::{KvError, Request, Result};

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
        // let mut response = String::new();
        // self.stream.read_to_string(&mut response).unwrap();
        let response: RESPType = serde_resp::from_reader(&mut self.stream)?;
        match response {
            RESPType::SimpleString(_) => Ok(()),
            RESPType::Error(err) => Err(KvError::Message(err)),
            _ => Err(KvError::Message("Unknown Error".to_owned()))
        }
    }

    pub fn get(&mut self, key: &str) -> Result<Option<String>> {
        let command: RESPType = Request::get(key).into();
        serde_resp::to_writer(&command, &mut self.stream).unwrap();
        let mut response = String::new();
        self.stream.read_to_string(&mut response).unwrap();
        let resp: RESPType = serde_resp::from_str(&response).unwrap();
        // currently only bulk str
        match resp {
            RESPType::BulkString(buf) => Ok(Some(String::from_utf8(buf).unwrap())),
            RESPType::Error(err) => Err(KvError::Message(err)),
            RESPType::SimpleString(msg) => Ok(Some(msg)),
            RESPType::None => Ok(None),
            RESPType::Array(arr) => Ok(Some(format!("{:?}", arr))),
            RESPType::Integer(int) => Ok(Some(int.to_string()))
        }
    }

    pub fn rm(&mut self, key: &str) -> Result<Option<String>> {
        let command: RESPType = Request::remove(key).into();
        serde_resp::to_writer(&command, &mut self.stream).unwrap();
        let mut response = String::new();
        self.stream.read_to_string(&mut response).unwrap();
        let response: RESPType = serde_resp::from_str(&response).unwrap();
        match response {
            RESPType::SimpleString(msg) => Ok(Some(msg)),
            RESPType::Error(err) => Err(KvError::Message(err)),
            _ => Err(KvError::Message("Unknown Error".to_owned()))
        }
    }
}