use std::collections::HashMap;
// https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use axum::extract::{MatchedPath, Request};
use pin_project::pin_project;
use tokio::time::Sleep;
use tokio_metrics::TaskMonitor;
use tower::Service;

#[derive(Debug, Clone)]
struct TokioTaskMetrics<S> {
    inner: S,
    task_monitors: HashMap<String, TaskMonitor>,
}

impl<S> TokioTaskMetrics<S> {
    fn new(inner: S) -> Self {
        TokioTaskMetrics {
            inner,
            task_monitors: Default::default(),
        }
    }
}

impl<S> Service<Request> for TokioTaskMetrics<S>
where
    S: Service<Request>,
    S::Error: Into<BoxError>,
{
    type Error = BoxError;
    type Future = ResponseFuture<tokio_metrics::Instrumented<S::Future>>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let path = request.extensions().get::<MatchedPath>().unwrap().as_str();
        let task_monitor = match self.task_monitors.get(path) {
            Some(task_monitor) => task_monitor,
            None => {
                let entry = self.task_monitors.entry(path.to_owned());
                entry.or_default()
            }
        };

        let response_future = task_monitor.instrument(self.inner.call(request));

        ResponseFuture { response_future }
    }
}

#[pin_project]
struct ResponseFuture<F> {
    #[pin]
    response_future: F,
}

impl<F, Response, Error> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response, Error>>,
    Error: Into<BoxError>,
{
    type Output = Result<Response, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.response_future.poll(cx) {
            Poll::Ready(result) => {
                let result = result.map_err(Into::into);
                return Poll::Ready(result);
            }
            Poll::Pending => {}
        }

        Poll::Pending
    }
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;
