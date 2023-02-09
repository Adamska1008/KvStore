mod naive;

use crate::Result;

pub trait ThreadPool {
    /// Creates a new thread pool, immediately spawning the specified number of
    /// threads.
    ///
    /// Return error if any thread fail to spawn, after which all previously-spawned
    /// threads are terminated.
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;

    /// Spawns a function into the thread pool.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce<()> + Send + 'static;
}
