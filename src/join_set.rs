use std::{
    fmt,
    future::Future,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};

use tokio::{sync::mpsc, task::JoinSet as TokioJoinSet};

use crate::{
    AbortHandle, Dispatcher, DispatcherRequest, DispatcherResponse, Handle, JoinError, LocalSet,
};

pub struct JoinSet<T> {
    num_tasks: AtomicUsize,
    request_sender: mpsc::UnboundedSender<DispatcherRequest<T>>,
    response_receiver: mpsc::UnboundedReceiver<DispatcherResponse<T>>,
}

impl<T> JoinSet<T> {
    pub fn new(concurrency: usize) -> Self
    where
        T: Send + 'static, // TODO: Remove send from here. This should not need to be send in the case of spawn local
    {
        let (request_sender, request_receiver) = mpsc::unbounded_channel();
        let (response_sender, response_receiver) = mpsc::unbounded_channel();

        let dispatcher = Dispatcher {
            concurrency,
            request_receiver,
            response_sender,
            join_set: TokioJoinSet::new(),
        };

        // TODO: confirm this gets cleaned up after join_set drop (unless detach_all is called)
        tokio::spawn(dispatcher.start());

        Self {
            num_tasks: AtomicUsize::new(0),
            request_sender,
            response_receiver,
        }
    }

    /// Returns all active tasks and all queued tasks
    /// TODO: add more counters so we can keep track of queued vs active vs completed
    pub fn len(&self) -> usize {
        self.num_tasks.load(Relaxed)
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
        // increment internal counter to keep track of len
        self.num_tasks.fetch_add(1, Relaxed);

        // Send future to channel
        let request = DispatcherRequest::new(task);
        // TODO: Error here means dispatcher is dead. Think about what this means. probably panic
        _ = self.request_sender.send(request);

        // TODO: construct abort handle
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
        if self.is_empty() {
            return None;
        }

        // wait on receive channel from dispatcher
        let response = self.response_receiver.recv().await?;

        // decrement length counter
        self.num_tasks.fetch_sub(1, Relaxed);

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
