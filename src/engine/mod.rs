pub mod kvstore;
pub mod sled;

pub use self::kvstore::KvStore;
pub use self::sled::Sled;

use crate::Result;

/// KvsEngine
///
/// For thread safety, must implement `Send` trait
///
/// The `Clone` trait is expected to be implemented like Arc::clone(), add reference while the source
/// keeps singularity
pub trait KvsEngine: Clone + Send + 'static {
    fn set(&self, key: &str, value: &str) -> Result<()>;
    fn get(&self, key: &str) -> Result<Option<String>>;
    fn remove(&self, key: &str) -> Result<Option<()>>;
}