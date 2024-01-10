use core::task::{Context, Poll};
use std::collections::HashMap;
// https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md
use std::future::Future;
use std::hash::{BuildHasher, Hash};
use std::pin::Pin;

use http::Method;
use opentelemetry::metrics::Unit;
use pin_project::pin_project;
use tokio_metrics::TaskMonitor;
use tower_layer::Layer;
use tower_service::Service;

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

#[allow(clippy::too_many_lines)]
pub fn tokio_runtime_metrics() {
    let meter = opentelemetry::global::meter("perfect-group-allocation");
    let handle = tokio::runtime::Handle::current();
    let runtime_monitor = tokio_metrics::RuntimeMonitor::new(&handle);

    let interval_root = std::sync::Mutex::new(runtime_monitor.intervals());

    let workers_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.workers_count")
        .init();
    let total_park_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.total_park_count")
        .init();
    let max_park_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.max_park_count")
        .init();
    let min_park_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.min_park_count")
        .init();
    let mean_poll_duration = meter
        .u64_observable_gauge("tokio.runtime_metrics.mean_poll_duration")
        .with_unit(Unit::new("ns"))
        .init();
    let mean_poll_duration_worker_min = meter
        .u64_observable_gauge("tokio.runtime_metrics.mean_poll_duration_worker_min")
        .with_unit(Unit::new("ns"))
        .init();
    let mean_poll_duration_worker_max = meter
        .u64_observable_gauge("tokio.runtime_metrics.mean_poll_duration_worker_max")
        .with_unit(Unit::new("ns"))
        .init();
    //let poll_count_histogram = meter.u64_histogram("test").init();
    let total_noop_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.total_noop_count")
        .init();
    let max_noop_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.max_noop_count")
        .init();
    let min_noop_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.min_noop_count")
        .init();
    let total_steal_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.total_steal_count")
        .init();
    let max_steal_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.max_steal_count")
        .init();
    let min_steal_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.min_steal_count")
        .init();
    let total_steal_operations = meter
        .u64_observable_gauge("tokio.runtime_metrics.total_steal_operations")
        .init();
    let max_steal_operations = meter
        .u64_observable_gauge("tokio.runtime_metrics.max_steal_operations")
        .init();
    let min_steal_operations = meter
        .u64_observable_gauge("tokio.runtime_metrics.min_steal_operations")
        .init();
    let num_remote_schedules = meter
        .u64_observable_gauge("tokio.runtime_metrics.num_remote_schedules")
        .init();
    let total_local_schedule_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.total_local_schedule_count")
        .init();
    let max_local_schedule_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.max_local_schedule_count")
        .init();
    let min_local_schedule_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.min_local_schedule_count")
        .init();
    let total_overflow_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.total_overflow_count")
        .init();
    let max_overflow_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.max_overflow_count")
        .init();
    let min_overflow_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.total_overflow_count")
        .init();
    let total_polls_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.total_polls_count")
        .init();
    let max_polls_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.max_polls_count")
        .init();
    let min_polls_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.min_polls_count")
        .init();
    let total_busy_duration = meter
        .u64_observable_gauge("tokio.runtime_metrics.total_busy_duration")
        .with_unit(Unit::new("ns"))
        .init();
    let max_busy_duration = meter
        .u64_observable_gauge("tokio.runtime_metrics.max_busy_duration")
        .with_unit(Unit::new("ns"))
        .init();
    let min_busy_duration = meter
        .u64_observable_gauge("tokio.runtime_metrics.min_busy_duration")
        .with_unit(Unit::new("ns"))
        .init();
    let injection_queue_depth = meter
        .u64_observable_gauge("tokio.runtime_metrics.injection_queue_depth")
        .init();
    let total_local_queue_depth = meter
        .u64_observable_gauge("tokio.runtime_metrics.total_local_queue_depth")
        .init();
    let max_local_queue_depth = meter
        .u64_observable_gauge("tokio.runtime_metrics.max_local_queue_depth")
        .init();
    let min_local_queue_depth = meter
        .u64_observable_gauge("tokio.runtime_metrics.min_local_queue_depth")
        .init();
    let elapsed = meter
        .u64_observable_gauge("tokio.runtime_metrics.elapsed")
        .with_unit(Unit::new("ns"))
        .init();
    let budget_forced_yield_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.budget_forced_yield_count")
        .init();
    let io_driver_ready_count = meter
        .u64_observable_gauge("tokio.runtime_metrics.io_driver_ready_count")
        .init();
    meter
        .register_callback(
            &[
                workers_count.as_any(),
                total_park_count.as_any(),
                max_park_count.as_any(),
                min_park_count.as_any(),
                mean_poll_duration.as_any(),
                mean_poll_duration_worker_min.as_any(),
                mean_poll_duration_worker_max.as_any(),
                //poll_count_histogram.as_any(),
                total_noop_count.as_any(),
                max_noop_count.as_any(),
                min_noop_count.as_any(),
                total_steal_count.as_any(),
                max_steal_count.as_any(),
                min_steal_count.as_any(),
                total_steal_operations.as_any(),
                max_steal_operations.as_any(),
                min_steal_operations.as_any(),
                num_remote_schedules.as_any(),
                total_local_schedule_count.as_any(),
                max_local_schedule_count.as_any(),
                min_local_schedule_count.as_any(),
                total_overflow_count.as_any(),
                max_overflow_count.as_any(),
                min_overflow_count.as_any(),
                total_polls_count.as_any(),
                max_polls_count.as_any(),
                min_polls_count.as_any(),
                total_busy_duration.as_any(),
                max_busy_duration.as_any(),
                min_busy_duration.as_any(),
                injection_queue_depth.as_any(),
                total_local_queue_depth.as_any(),
                max_local_queue_depth.as_any(),
                min_local_queue_depth.as_any(),
                elapsed.as_any(),
                budget_forced_yield_count.as_any(),
                io_driver_ready_count.as_any(),
            ],
            move |observer| {
                let runtime_metrics = interval_root.lock().unwrap().next().unwrap();
                let attrs = &[];
                observer.observe_u64(
                    &workers_count,
                    runtime_metrics.workers_count.try_into().unwrap(),
                    attrs,
                );
                observer.observe_u64(&total_park_count, runtime_metrics.total_park_count, attrs);
                observer.observe_u64(&max_park_count, runtime_metrics.max_park_count, attrs);
                observer.observe_u64(&min_park_count, runtime_metrics.min_park_count, attrs);
                observer.observe_u64(
                    &mean_poll_duration,
                    runtime_metrics
                        .mean_poll_duration
                        .as_nanos()
                        .try_into()
                        .unwrap(),
                    attrs,
                );
                observer.observe_u64(
                    &mean_poll_duration_worker_min,
                    runtime_metrics
                        .mean_poll_duration_worker_min
                        .as_nanos()
                        .try_into()
                        .unwrap(),
                    attrs,
                );
                observer.observe_u64(
                    &mean_poll_duration_worker_max,
                    runtime_metrics
                        .mean_poll_duration_worker_max
                        .as_nanos()
                        .try_into()
                        .unwrap(),
                    attrs,
                );
                // TODO poll_count_histogram
                observer.observe_u64(&total_noop_count, runtime_metrics.total_noop_count, attrs);
                observer.observe_u64(&max_noop_count, runtime_metrics.max_noop_count, attrs);
                observer.observe_u64(&min_noop_count, runtime_metrics.min_noop_count, attrs);
                observer.observe_u64(&total_steal_count, runtime_metrics.total_steal_count, attrs);
                observer.observe_u64(&max_steal_count, runtime_metrics.max_steal_count, attrs);
                observer.observe_u64(&min_steal_count, runtime_metrics.min_steal_count, attrs);
                observer.observe_u64(
                    &total_steal_operations,
                    runtime_metrics.total_steal_operations,
                    attrs,
                );
                observer.observe_u64(
                    &max_steal_operations,
                    runtime_metrics.max_steal_operations,
                    attrs,
                );
                observer.observe_u64(
                    &min_steal_operations,
                    runtime_metrics.min_steal_operations,
                    attrs,
                );
                observer.observe_u64(
                    &num_remote_schedules,
                    runtime_metrics.num_remote_schedules,
                    attrs,
                );
                observer.observe_u64(
                    &total_local_schedule_count,
                    runtime_metrics.total_local_schedule_count,
                    attrs,
                );
                observer.observe_u64(
                    &max_local_schedule_count,
                    runtime_metrics.max_local_schedule_count,
                    attrs,
                );
                observer.observe_u64(
                    &min_local_schedule_count,
                    runtime_metrics.min_local_schedule_count,
                    attrs,
                );
                observer.observe_u64(
                    &total_overflow_count,
                    runtime_metrics.total_overflow_count,
                    attrs,
                );
                observer.observe_u64(
                    &max_overflow_count,
                    runtime_metrics.max_overflow_count,
                    attrs,
                );
                observer.observe_u64(
                    &min_overflow_count,
                    runtime_metrics.min_overflow_count,
                    attrs,
                );
                observer.observe_u64(&total_polls_count, runtime_metrics.total_polls_count, attrs);
                observer.observe_u64(&max_polls_count, runtime_metrics.max_polls_count, attrs);
                observer.observe_u64(&min_polls_count, runtime_metrics.min_polls_count, attrs);
                observer.observe_u64(
                    &total_busy_duration,
                    runtime_metrics
                        .total_busy_duration
                        .as_nanos()
                        .try_into()
                        .unwrap(),
                    attrs,
                );
                observer.observe_u64(
                    &max_busy_duration,
                    runtime_metrics
                        .max_busy_duration
                        .as_nanos()
                        .try_into()
                        .unwrap(),
                    attrs,
                );
                observer.observe_u64(
                    &min_busy_duration,
                    runtime_metrics
                        .min_busy_duration
                        .as_nanos()
                        .try_into()
                        .unwrap(),
                    attrs,
                );
                observer.observe_u64(
                    &injection_queue_depth,
                    runtime_metrics.injection_queue_depth.try_into().unwrap(),
                    attrs,
                );
                observer.observe_u64(
                    &total_local_queue_depth,
                    runtime_metrics.total_local_queue_depth.try_into().unwrap(),
                    attrs,
                );
                observer.observe_u64(
                    &max_local_queue_depth,
                    runtime_metrics.max_local_queue_depth.try_into().unwrap(),
                    attrs,
                );
                observer.observe_u64(
                    &min_local_queue_depth,
                    runtime_metrics.min_local_queue_depth.try_into().unwrap(),
                    attrs,
                );
                observer.observe_u64(
                    &elapsed,
                    runtime_metrics.elapsed.as_nanos().try_into().unwrap(),
                    attrs,
                );
                observer.observe_u64(
                    &budget_forced_yield_count,
                    runtime_metrics.budget_forced_yield_count,
                    attrs,
                );
                observer.observe_u64(
                    &io_driver_ready_count,
                    runtime_metrics.io_driver_ready_count,
                    attrs,
                );
            },
        )
        .unwrap();
}
