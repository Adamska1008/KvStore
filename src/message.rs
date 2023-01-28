use serde_resp::{array, bulk, err, none, RESPType, simple};

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
    fn into(self) -> RESPType {
        match self {
            Request::Set { key, value } => array!(bulk!("set"), bulk!(key), bulk!(value)),
            Request::Get { key } => array!(bulk!("get"), bulk!(key)),
            Request::Remove { key } => array!(bulk!("rm"), bulk!(key)),
        }
    }
}

/// May deserialize as:
/// `RESPType::BulkString(str)`
/// `RESPType::None`
/// `RESPType::Error(err)`
pub enum GetResponse {
    Ok(Option<String>),
    Err(String)
}

impl Into<RESPType> for GetResponse {
    fn into(self) -> RESPType {
        match self {
            GetResponse::Ok(opt_str) => match opt_str {
                Some(str) => bulk!(str),
                None => none!()
            },
            GetResponse::Err(err) => err!(err)
        }
    }
}

/// May deserialize as:
/// `RESPType::SimpleString("OK")`
/// `RESPType::Error(err)`
pub enum SetResponse {
    Ok(()),
    Err(String)
}

impl Into<RESPType> for SetResponse {
    fn into(self) -> RESPType {
        match self {
            SetResponse::Ok(()) => simple!("OK"),
            SetResponse::Err(err) => err!(err)
        }
    }
}

/// May deserialize as:
/// `RESPType::SimpleString("OK")`
/// `RESPType::None`
/// `RESPType::Error(err)`
pub enum RemoveResponse {
    Ok(Option<()>),
    Err(String)
}

impl Into<RESPType> for RemoveResponse {
    fn into(self) -> RESPType {
        match self {
            RemoveResponse::Ok(opt) => match opt {
                Some(()) => simple!("OK"),
                None => none!()
            }
            RemoveResponse::Err(err) => err!(err)
        }
    }
}
