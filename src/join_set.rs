use std::{
    fmt,
    future::Future,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
    sync::Arc,
};

use tokio::{
    sync::{mpsc, Semaphore},
    task::JoinSet as TokioJoinSet,
};

use crate::{AbortHandle, DispatcherResponse, Handle, JoinError, LocalSet};

pub struct JoinSet<T> {
    num_inactive_tasks: Arc<AtomicUsize>,
    num_active_tasks: Arc<AtomicUsize>,
    response_sender: mpsc::UnboundedSender<DispatcherResponse<T>>,
    response_receiver: mpsc::UnboundedReceiver<DispatcherResponse<T>>,
    active_semaphore: Arc<Semaphore>,
    inner_join_set: TokioJoinSet<()>,
}

impl<T> JoinSet<T> {
    pub fn new(concurrency: usize) -> Self {
        let (response_sender, response_receiver) = mpsc::unbounded_channel();

        Self {
            num_inactive_tasks: Arc::new(AtomicUsize::new(0)),
            num_active_tasks: Arc::new(AtomicUsize::new(0)),
            response_sender,
            response_receiver,
            inner_join_set: TokioJoinSet::new(),
            active_semaphore: Arc::new(Semaphore::new(concurrency)),
        }
    }

    /// Returns all active tasks and all queued tasks
    pub fn len(&self) -> usize {
        self.num_active_tasks.load(Relaxed) + self.num_inactive_tasks.load(Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<T: 'static> JoinSet<T> {
    pub fn spawn<F>(&mut self, task: F) -> AbortHandle
    where
        F: Future<Output = T> + Send + 'static,
        T: Send,
    {
        self.num_inactive_tasks.fetch_add(1, Relaxed);

        let task_semaphore = self.active_semaphore.clone();
        let task_inactive_count = self.num_inactive_tasks.clone();
        let task_active_count = self.num_active_tasks.clone();
        let task_response_channel = self.response_sender.clone();

        let wrapped_task = async move {
            // SAFETY: error here means the semaphore is closed which is currently not logically possible
            let _permit = task_semaphore.acquire_owned().await.unwrap();

            // TODO: consider a drop mechanism for these atomics for good cleanup on panic
            task_inactive_count.fetch_sub(1, Relaxed);
            task_active_count.fetch_add(1, Relaxed);

            let output = task.await;

            // TODO: confirm that ignoring this error is okay
            _ = task_response_channel.send(DispatcherResponse { payload: output });

            task_active_count.fetch_sub(1, Relaxed);
        };

        self.inner_join_set.spawn(wrapped_task);

        // TODO: might be able to just use regular abort handle
        AbortHandle
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
        self.num_inactive_tasks.fetch_add(1, Relaxed);

        let task_semaphore = self.active_semaphore.clone();
        let task_inactive_count = self.num_inactive_tasks.clone();
        let task_active_count = self.num_active_tasks.clone();
        let task_response_channel = self.response_sender.clone();

        let wrapped_task = async move {
            // SAFETY: error here means the semaphore is closed which is currently not logically possible
            let _permit = task_semaphore.acquire_owned().await.unwrap();

            // TODO: consider a drop mechanism for these atomics for good cleanup on panic
            task_inactive_count.fetch_sub(1, Relaxed);
            task_active_count.fetch_add(1, Relaxed);

            let output = task.await;

            // TODO: confirm that ignoring this error is okay
            _ = task_response_channel.send(DispatcherResponse { payload: output });

            task_active_count.fetch_sub(1, Relaxed);
        };

        self.inner_join_set.spawn_local(wrapped_task);

        //TODO: might be able to do normal abort handle
        AbortHandle
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
        if self.is_empty() {
            return None;
        }

        // wait on receive channel from dispatcher
        let response = self.response_receiver.recv().await?;

        // // decrement length counter
        // self.num_tasks.fetch_sub(1, Relaxed);

        Some(Ok(response.payload))
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
