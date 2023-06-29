use std::{
    fmt,
    future::Future,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
    sync::Arc,
};

use tokio::{sync::Semaphore, task::AbortHandle, task::JoinSet as TokioJoinSet};

use crate::{Handle, JoinError, LocalSet};

pub struct JoinSet<T> {
    num_inactive_tasks: Arc<AtomicUsize>,
    num_active_tasks: Arc<AtomicUsize>,
    active_semaphore: Arc<Semaphore>,
    inner_join_set: TokioJoinSet<T>,
}

impl<T> JoinSet<T> {
    pub fn new(concurrency: usize) -> Self {
        Self {
            num_inactive_tasks: Arc::new(AtomicUsize::new(0)),
            num_active_tasks: Arc::new(AtomicUsize::new(0)),
            inner_join_set: TokioJoinSet::new(),
            active_semaphore: Arc::new(Semaphore::new(concurrency)),
        }
    }

    /// Returns all active tasks and all queued tasks
    pub fn len(&self) -> usize {
        // TODO: add a third atomic and use all three here
        self.inner_join_set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    //TODO: add functions for getting num active, num inactive, num complete
}
impl<T: 'static> JoinSet<T> {
    fn wrap_task<F>(&self, task: F) -> impl Future<Output = T> + 'static
    where
        F: Future<Output = T> + 'static,
    {
        self.num_inactive_tasks.fetch_add(1, Relaxed);

        let task_semaphore = self.active_semaphore.clone();
        let task_inactive_count = self.num_inactive_tasks.clone();
        let task_active_count = self.num_active_tasks.clone();

        async move {
            // SAFETY: error here means the semaphore is closed which is currently not logically possible
            let _permit = task_semaphore.acquire_owned().await.unwrap();

            // TODO: consider a drop mechanism for these atomics for good cleanup on panic
            task_inactive_count.fetch_sub(1, Relaxed);
            task_active_count.fetch_add(1, Relaxed);

            let output = task.await;

            task_active_count.fetch_sub(1, Relaxed);

            output
        }
    }

    fn wrap_blocking_task<F>(&self, task: F) -> impl FnOnce() -> T + Send + 'static
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send,
    {
        self.num_inactive_tasks.fetch_add(1, Relaxed);

        let task_semaphore = self.active_semaphore.clone();
        let task_inactive_count = self.num_inactive_tasks.clone();
        let task_active_count = self.num_active_tasks.clone();

        move || {
            // SAFETY: error here means the semaphore is closed which is currently not logically possible
            let _permit = task_semaphore.try_acquire_owned().unwrap(); //TODO: this does not work!!!!!! Need another way. Either spin loop or some blocking notifier

            // TODO: consider a drop mechanism for these atomics for good cleanup on panic
            task_inactive_count.fetch_sub(1, Relaxed);
            task_active_count.fetch_add(1, Relaxed);

            let output = task();

            task_active_count.fetch_sub(1, Relaxed);

            output
        }
    }

    pub fn spawn<F>(&mut self, task: F) -> AbortHandle
    where
        F: Future<Output = T> + Send + 'static,
        T: Send,
    {
        self.inner_join_set.spawn(self.wrap_task(task))
    }

    pub fn spawn_on<F>(&mut self, task: F, handle: &Handle) -> AbortHandle
    where
        F: Future<Output = T> + Send + 'static,
        T: Send,
    {
        self.inner_join_set.spawn_on(self.wrap_task(task), handle)
    }

    pub fn spawn_local<F>(&mut self, task: F) -> AbortHandle
    where
        F: Future<Output = T> + 'static,
    {
        self.inner_join_set.spawn_local(self.wrap_task(task))
    }

    pub fn spawn_local_on<F>(&mut self, task: F, local_set: &LocalSet) -> AbortHandle
    where
        F: Future<Output = T> + 'static,
    {
        self.inner_join_set
            .spawn_local_on(self.wrap_task(task), local_set)
    }

    pub fn spawn_blocking<F>(&mut self, f: F) -> AbortHandle
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send,
    {
        self.inner_join_set
            .spawn_blocking(self.wrap_blocking_task(f))
    }

    pub fn spawn_blocking_on<F>(&mut self, f: F, handle: &Handle) -> AbortHandle
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send,
    {
        self.inner_join_set
            .spawn_blocking_on(self.wrap_blocking_task(f), handle)
    }

    pub async fn join_next(&mut self) -> Option<Result<T, JoinError>> {
        self.inner_join_set.join_next().await
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

impl<T> Drop for JoinSet<T> {
    fn drop(&mut self) {
        //TODO:
    }
}

impl<T> Default for JoinSet<T> {
    fn default() -> Self {
        unimplemented!()
    }
}

impl<T> fmt::Debug for JoinSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: add active tasks and queued tasks here
        f.debug_struct("JoinSet").field("len", &self.len()).finish()
    }
}
