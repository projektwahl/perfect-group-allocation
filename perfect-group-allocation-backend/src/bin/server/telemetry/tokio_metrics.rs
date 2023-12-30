use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::Infallible;
// https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md
use std::fmt;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{Arc, RwLock, Weak};
use std::task::{Context, Poll};
use std::time::Duration;

use axum::extract::{MatchedPath, Request};
use crossbeam::atomic::AtomicCell;
use opentelemetry::metrics::Unit;
use opentelemetry::KeyValue;
use pin_project::pin_project;
use tokio::time::Sleep;
use tokio_metrics::TaskMonitor;
use tower::{Layer, Service};
use tracing::{debug, error};

// we should really initialize at start

#[derive(Clone)]
pub struct TokioTaskMetricsLayer {
    task_monitors: Arc<AtomicCell<Arc<HashMap<String, TaskMonitor>>>>,
}

impl TokioTaskMetricsLayer {
    pub fn new() -> Self {
        Self {
            task_monitors: Default::default(),
        }
    }
}

impl<S> Layer<S> for TokioTaskMetricsLayer {
    type Service = TokioTaskMetrics<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TokioTaskMetrics {
            inner,
            task_monitors: Arc::downgrade(&self.task_monitors),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokioTaskMetrics<S> {
    inner: S,
    task_monitors: Weak<RwLock<HashMap<String, TaskMonitor>>>,
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

        let task_monitor = {
            let arc = self.task_monitors.upgrade().unwrap();
            let lock = arc.read().unwrap();
            if let Some(task_monitor) = lock.get(path) {
                task_monitor.clone()
            } else {
                drop(lock);
                let path = path.to_owned();
                let entry = arc.write().unwrap().entry(path.clone());
                let new_task_monitor = entry.or_default();
                let meter = opentelemetry::global::meter("perfect-group-allocation");
                let interval_root = std::sync::Mutex::new(new_task_monitor.intervals());
                let path = Arc::new(path);

                let mean_poll_duration = meter
                    .u64_observable_gauge("tokio.task_monitor.mean_poll_duration")
                    .with_unit(Unit::new("ns"))
                    .init();
                let slow_poll_ratio = meter
                    .f64_observable_gauge("tokio.task_monitor.slow_poll_ratio")
                    .init();

                meter
                    .register_callback(
                        &[mean_poll_duration.as_any(), slow_poll_ratio.as_any()],
                        move |observer| {
                            // TODO FIXME get and post?
                            debug!("metrics for {}", path);
                            let task_metrics = interval_root.lock().unwrap().next().unwrap();
                            observer.observe_u64(
                                &mean_poll_duration,
                                task_metrics.mean_poll_duration().subsec_nanos().into(),
                                &[KeyValue::new(
                                    opentelemetry_semantic_conventions::trace::URL_PATH,
                                    path.deref().clone(),
                                )],
                            );
                            observer.observe_f64(
                                &slow_poll_ratio,
                                task_metrics.slow_poll_ratio(),
                                &[KeyValue::new(
                                    opentelemetry_semantic_conventions::trace::URL_PATH,
                                    path.deref().clone(),
                                )],
                            );
                        },
                    )
                    .unwrap();

                new_task_monitor.clone()
            }
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
