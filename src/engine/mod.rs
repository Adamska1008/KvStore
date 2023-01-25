mod command;
mod io;
mod store;
mod tools;

pub use command::Command;
pub use store::KvStore;

use crate::error::Result;

pub trait KvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn remove(&mut self, key: String) -> Result<()>;
}