pub mod command;
pub mod error;
pub mod io;
pub mod store;
pub mod tools;

pub use command::Command;
pub use error::{KvError, Result};
pub use store::KvStore;

pub trait KvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn get(&mut self, key: String) -> Result<Option<String>>;
    fn remove(&mut self, key: String) -> Result<()>;
}