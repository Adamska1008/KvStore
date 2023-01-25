mod kvstore;

pub use kvstore::KvStore;

use crate::Result;

pub trait KvsEngine {
    fn set(&mut self, key: &str, value: &str) -> Result<()>;
    fn get(&mut self, key: &str) -> Result<Option<String>>;
    fn remove(&mut self, key: &str) -> Result<Option<()>>;
}