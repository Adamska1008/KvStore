use std::thread;
use crate::thread_pool::ThreadPool;

pub struct NaiveThreadPool;

/// The implementation of thread pool does nothing, keeps none threads and immediately spawn the given job.
impl ThreadPool for NaiveThreadPool {
    fn new(_: u32) -> crate::Result<Self> where Self: Sized {
        Ok(Self)
    }

    fn spawn<F>(&self, job: F) where F: FnOnce<()> + Send + 'static {
        thread::spawn(job);
    }
}