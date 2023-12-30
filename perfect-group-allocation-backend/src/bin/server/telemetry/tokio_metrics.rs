use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::Infallible;
// https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md
use std::fmt;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use axum::extract::{MatchedPath, Request};
use opentelemetry::KeyValue;
use pin_project::pin_project;
use tokio::time::Sleep;
use tokio_metrics::TaskMonitor;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct TokioTaskMetricsLayer;

impl<S> Layer<S> for TokioTaskMetricsLayer {
    type Service = TokioTaskMetrics<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TokioTaskMetrics::new(inner)
    }
}

#[derive(Debug, Clone)]
pub struct TokioTaskMetrics<S> {
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
{
    type Error = S::Error;
    type Future = ResponseFuture<tokio_metrics::Instrumented<S::Future>>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let path = request.extensions().get::<MatchedPath>().unwrap().as_str();
        let task_monitor = if let Some(task_monitor) = self.task_monitors.get(path) {
            task_monitor
        } else {
            let path = path.to_owned();
            let entry = self.task_monitors.entry(path.clone());
            let new_task_monitor = entry.or_default();
            let meter = opentelemetry::global::meter("perfect-group-allocation");
            let interval_root = std::sync::Mutex::new(new_task_monitor.intervals());
            let path = Arc::new(path);
            meter
                .u64_observable_gauge("tokio.task_monitor.mean_poll_duration")
                .with_callback(move |gauge| {
                    gauge.observe(
                        interval_root
                            .lock()
                            .unwrap()
                            .next()
                            .unwrap()
                            .mean_poll_duration()
                            .subsec_nanos()
                            .into(),
                        &[KeyValue::new(
                            opentelemetry_semantic_conventions::trace::URL_PATH,
                            path.deref().clone(),
                        )],
                    );
                })
                .init();

            new_task_monitor
        };

        let response_future = task_monitor.instrument(self.inner.call(request));

        ResponseFuture { response_future }
    }
}

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    response_future: F,
}

impl<F, Response, Error> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response, Error>>,
{
    type Output = Result<Response, Error>;

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
