use core::task::{Context, Poll};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::Infallible;
// https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md
use std::fmt;
use std::future::Future;
use std::hash::{BuildHasher, Hash, Hasher};
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;

use axum::extract::{MatchedPath, Request};
use crossbeam::atomic::AtomicCell;
use http::Method;
use opentelemetry::metrics::Unit;
use opentelemetry::KeyValue;
use pin_project::pin_project;
use tokio::time::Sleep;
use tokio_metrics::TaskMonitor;
use tower::{Layer, Service};
use tracing::{debug, error};

// TODO runtime metrics
// https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html
// https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.RuntimeMetrics.html#structfield.max_steal_count

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct BorrowedMethodAndPath<'a> {
    pub method: &'a Method,
    pub path: &'a str,
}

#[derive(Clone)]
pub struct TokioTaskMetricsLayer {
    pub task_monitors: HashMap<BorrowedMethodAndPath<'static>, TaskMonitor>,
}

impl<S> Layer<S> for TokioTaskMetricsLayer {
    type Service = TokioTaskMetrics<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TokioTaskMetrics {
            inner,
            task_monitors: self.task_monitors.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokioTaskMetrics<S> {
    inner: S,
    task_monitors: HashMap<BorrowedMethodAndPath<'static>, TaskMonitor>,
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
        let method = request.method();
        let path = request.extensions().get::<MatchedPath>().unwrap().as_str();

        let key = BorrowedMethodAndPath { method, path };
        if let Some((_k, task_monitor)) = self
            .task_monitors
            .raw_entry()
            .from_hash(self.task_monitors.hasher().hash_one(&key), |k| key == *k)
        {
            let response_future = task_monitor.instrument(self.inner.call(request));

            ResponseFuture { response_future }
        } else {
            unreachable!();
        }
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
        this.response_future.poll(cx)
    }
}
