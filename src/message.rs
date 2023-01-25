use serde_resp::{array, bulk, RESPType};

#[derive(Debug)]
pub enum Request {
    Set { key: String, value: String},
    Get { key: String },
    Remove { key: String }
}

impl Request {
    pub fn set(key: &str, value: &str) -> Self {
        Request::Set {
            key: key.to_owned(),
            value: value.to_owned()
        }
    }
    pub fn get(key: &str) -> Self {
        Request::Get {
            key: key.to_owned()
        }
    }
    pub fn remove(key: &str) -> Self {
        Request::Remove {
            key: key.to_owned()
        }
    }
}

impl Into<RESPType> for Request {
    fn into(&self) -> RESPType {
        match self {
            Request::Set { key, value } => {
                array!(bulk!("set"), bulk!(key), bulk!("value"))
            },
            Request::Get { key } => {
                array!(bulk!("get"), bulk!(key))
            },
            Request::Remove { key } => {
                array!(bulk!("rm"), bulk!(key))
            }
        }
    }
}
