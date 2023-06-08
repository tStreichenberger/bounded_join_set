use std::{future::Future, marker::PhantomData};

use crate::{AbortHandle, Handle, JoinError, LocalSet};

pub struct JoinSet<T>(PhantomData<T>);

impl<T> JoinSet<T> {
    pub fn new(concurrency: usize) -> Self {
        unimplemented!()
    }

    /// Returns all active tasks and all queues tasks
    pub fn len(&self) -> usize {
        unimplemented!()
    }

    pub fn is_empty(&self) -> bool {
        unimplemented!()
    }
}

impl<T: 'static> JoinSet<T> {
    pub fn spawn<F>(&mut self, task: F) -> AbortHandle
    where
        F: Future<Output = T> + Send + 'static,
        T: Send,
    {
        unimplemented!()
    }

    pub fn spawn_on<F>(&mut self, task: F, handle: &Handle) -> AbortHandle
    where
        F: Future<Output = T> + Send + 'static,
        T: Send,
    {
        unimplemented!()
    }

    pub fn spawn_local<F>(&mut self, task: F) -> AbortHandle
    where
        F: Future<Output = T> + 'static,
    {
        unimplemented!()
    }

    pub fn spawn_local_on<F>(&mut self, task: F, local_set: &LocalSet) -> AbortHandle
    where
        F: Future<Output = T> + 'static,
    {
        unimplemented!()
    }

    pub fn spawn_blocking<F>(&mut self, f: F) -> AbortHandle
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send,
    {
        unimplemented!()
    }

    pub fn spawn_blocking_on<F>(&mut self, f: F, handle: &Handle) -> AbortHandle
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send,
    {
        unimplemented!()
    }

    pub async fn join_next(&mut self) -> Option<Result<T, JoinError>> {
        unimplemented!()
    }

    pub async fn shutdown(&mut self) {
        unimplemented!()
    }

    pub fn abort_all(&mut self) {
        unimplemented!()
    }

    pub fn detach_all(&mut self) {
        unimplemented!()
    }
}
