use std::future::Future;
use std::pin::Pin;

use tokio::sync::mpsc;
use tokio::task::JoinSet as TokioJoinSet;

pub struct DispatcherRequest<T> {
    fut: Pin<Box<dyn Future<Output = T> + Send + 'static>>,
}

impl<T> DispatcherRequest<T> {
    pub fn new<F>(task: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        Self {
            fut: Box::pin(task),
        }
    }
}

pub struct DispatcherResponse<T> {
    pub payload: T,
}

pub struct Dispatcher<T> {
    pub request_receiver: mpsc::UnboundedReceiver<DispatcherRequest<T>>,
    pub response_sender: mpsc::UnboundedSender<DispatcherResponse<T>>,
    // maybe atomic if we want to allow for changing on the fly
    pub concurrency: usize,
    pub join_set: TokioJoinSet<()>,
}

// just slapping send on here for now but this will be a problem on spawn local
// this is going to get much more complicated with that
//
// maybe put the dispatch function as a method of the request and make the request an enum based on the response type
impl<T: 'static + Send> Dispatcher<T> {
    pub async fn start(mut self) {
        while let Some(request) = self.request_receiver.recv().await {
            // check concurrency
            while self.join_set.len() >= self.concurrency {
                // TODO: double check these unwraps are valid
                self.join_set.join_next().await.unwrap().unwrap();
            }

            self.dispatch(request);
        }
    }

    pub fn dispatch(&mut self, request: DispatcherRequest<T>) {
        let responder = self.response_sender.clone();

        let task = async move {
            let payload = request.fut.await;

            let response = DispatcherResponse { payload };
            // TODO: confirm that ignoring this error is okay
            _ = responder.send(response);
        };

        // TODO: do something with the abort handle from this. Perhaps inner hash map in JoinSet
        self.join_set.spawn(task);
    }
}
