mod kvstore;
mod sled;

pub use self::kvstore::KvStore;
pub use self::sled::Sled;

use crate::Result;

pub trait KvsEngine {
    fn set(&mut self, key: &str, value: &str) -> Result<()>;
    fn get(&mut self, key: &str) -> Result<Option<String>>;
    fn remove(&mut self, key: &str) -> Result<Option<()>>;
}