use std::io::Read;
use std::net::TcpStream;
use serde_resp::RESPType;

pub fn unwrap_bulk_str(resp: &RESPType) -> String {
    if let RESPType::BulkString(bulk_str) = resp {
        String::from_utf8(bulk_str.clone()).unwrap()
    } else {
        panic!("not a resp bulk str")
    }
}

pub fn read_to_end(stream: &mut TcpStream) -> String {
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