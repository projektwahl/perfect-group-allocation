use std::collections::HashMap;

use axum::handler::Handler;
use axum::Router;
use http::Method;
use opentelemetry::metrics::Unit;
use opentelemetry::KeyValue;
use tokio_metrics::TaskMonitor;

use crate::telemetry::tokio_metrics::{BorrowedMethodAndPath, TokioTaskMetricsLayer};
use crate::MyState;

#[derive(Default)]
pub struct MyRouter {
    router: Router<MyState>,
    task_monitors: HashMap<BorrowedMethodAndPath<'static>, TaskMonitor>,
}

impl MyRouter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[track_caller]
    #[must_use]
    pub fn route<T: 'static, H: Handler<T, MyState>>(
        mut self,
        method: &'static Method,
        path: &'static str,
        handler: H,
    ) -> Self {
        let meter = opentelemetry::global::meter("perfect-group-allocation");
        let mut new_task_monitor = TaskMonitor::builder();
        new_task_monitor
            .with_long_delay_threshold(TaskMonitor::DEFAULT_LONG_DELAY_THRESHOLD * 2)
            .with_slow_poll_threshold(TaskMonitor::DEFAULT_SLOW_POLL_THRESHOLD * 2);
        let new_task_monitor = new_task_monitor.build();

        let interval_root = std::sync::Mutex::new(new_task_monitor.intervals());

        let dropped_count = meter
            .u64_observable_gauge("tokio.task_metrics.dropped_count")
            .init();
        let first_poll_count = meter
            .u64_observable_gauge("tokio.task_metrics.first_poll_count")
            .init();
        let instrumented_count = meter
            .u64_observable_gauge("tokio.task_metrics.instrumented_count")
            .init();
        let total_fast_poll_count = meter
            .u64_observable_gauge("tokio.task_metrics.total_fast_poll_count")
            .init();
        let total_fast_poll_duration = meter
            .u64_observable_gauge("tokio.task_metrics.total_fast_poll_duration")
            .with_unit(Unit::new("ns"))
            .init();
        let total_first_poll_delay = meter
            .u64_observable_gauge("tokio.task_metrics.total_first_poll_delay")
            .with_unit(Unit::new("ns"))
            .init();
        let total_idle_duration = meter
            .u64_observable_gauge("tokio.task_metrics.total_idle_duration")
            .with_unit(Unit::new("ns"))
            .init();
        let total_idled_count = meter
            .u64_observable_gauge("tokio.task_metrics.total_idled_count")
            .init();
        let total_long_delay_count = meter
            .u64_observable_gauge("tokio.task_metrics.total_long_delay_count")
            .init();
        let total_long_delay_duration = meter
            .u64_observable_gauge("tokio.task_metrics.total_long_delay_duration")
            .with_unit(Unit::new("ns"))
            .init();
        let total_poll_count = meter
            .u64_observable_gauge("tokio.task_metrics.total_poll_count")
            .init();
        let total_poll_duration = meter
            .u64_observable_gauge("tokio.task_metrics.total_poll_duration")
            .with_unit(Unit::new("ns"))
            .init();
        let total_scheduled_count = meter
            .u64_observable_gauge("tokio.task_metrics.total_scheduled_count")
            .init();
        let total_scheduled_duration = meter
            .u64_observable_gauge("tokio.task_metrics.total_scheduled_duration")
            .with_unit(Unit::new("ns"))
            .init();
        let total_short_delay_count = meter
            .u64_observable_gauge("tokio.task_metrics.total_short_delay_count")
            .init();
        let total_short_delay_duration = meter
            .u64_observable_gauge("tokio.task_metrics.total_short_delay_duration")
            .with_unit(Unit::new("ns"))
            .init();
        let total_slow_poll_count = meter
            .u64_observable_gauge("tokio.task_metrics.total_slow_poll_count")
            .init();
        let total_slow_poll_duration = meter
            .u64_observable_gauge("tokio.task_metrics.total_slow_poll_duration")
            .with_unit(Unit::new("ns"))
            .init();
        let attrs = [
            KeyValue::new(
                opentelemetry_semantic_conventions::trace::HTTP_REQUEST_METHOD,
                method.as_str(),
            ),
            KeyValue::new(opentelemetry_semantic_conventions::trace::URL_PATH, path),
            KeyValue::new(
                opentelemetry_semantic_conventions::trace::HTTP_ROUTE,
                method.as_str().to_owned() + path,
            ),
        ];
        meter
            .register_callback(
                &[
                    dropped_count.as_any(),
                    first_poll_count.as_any(),
                    instrumented_count.as_any(),
                    total_fast_poll_count.as_any(),
                    total_fast_poll_duration.as_any(),
                    total_first_poll_delay.as_any(),
                    total_idle_duration.as_any(),
                    total_idled_count.as_any(),
                    total_long_delay_count.as_any(),
                    total_long_delay_duration.as_any(),
                    total_poll_count.as_any(),
                    total_poll_duration.as_any(),
                    total_scheduled_count.as_any(),
                    total_scheduled_duration.as_any(),
                    total_short_delay_count.as_any(),
                    total_short_delay_duration.as_any(),
                    total_slow_poll_count.as_any(),
                    total_slow_poll_duration.as_any(),
                ],
                move |observer| {
                    let task_metrics = interval_root.lock().unwrap().next().unwrap();

                    observer.observe_u64(&dropped_count, task_metrics.dropped_count, &attrs);
                    observer.observe_u64(&first_poll_count, task_metrics.first_poll_count, &attrs);
                    observer.observe_u64(
                        &instrumented_count,
                        task_metrics.instrumented_count,
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_fast_poll_count,
                        task_metrics.total_fast_poll_count,
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_fast_poll_duration,
                        task_metrics
                            .total_fast_poll_duration
                            .as_nanos()
                            .try_into()
                            .unwrap(),
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_first_poll_delay,
                        task_metrics
                            .total_first_poll_delay
                            .as_nanos()
                            .try_into()
                            .unwrap(),
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_idle_duration,
                        task_metrics
                            .total_idle_duration
                            .as_nanos()
                            .try_into()
                            .unwrap(),
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_idled_count,
                        task_metrics.total_idled_count,
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_long_delay_count,
                        task_metrics.total_long_delay_count,
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_long_delay_duration,
                        task_metrics
                            .total_long_delay_duration
                            .as_nanos()
                            .try_into()
                            .unwrap(),
                        &attrs,
                    );
                    observer.observe_u64(&total_poll_count, task_metrics.total_poll_count, &attrs);
                    observer.observe_u64(
                        &total_poll_duration,
                        task_metrics
                            .total_poll_duration
                            .as_nanos()
                            .try_into()
                            .unwrap(),
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_scheduled_count,
                        task_metrics.total_scheduled_count,
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_scheduled_duration,
                        task_metrics
                            .total_scheduled_duration
                            .as_nanos()
                            .try_into()
                            .unwrap(),
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_short_delay_count,
                        task_metrics.total_short_delay_count,
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_short_delay_duration,
                        task_metrics
                            .total_short_delay_duration
                            .as_nanos()
                            .try_into()
                            .unwrap(),
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_slow_poll_count,
                        task_metrics.total_slow_poll_count,
                        &attrs,
                    );
                    observer.observe_u64(
                        &total_slow_poll_duration,
                        task_metrics
                            .total_slow_poll_duration
                            .as_nanos()
                            .try_into()
                            .unwrap(),
                        &attrs,
                    );
                },
            )
            .unwrap();
        assert!(
            self.task_monitors
                .insert(BorrowedMethodAndPath { method, path }, new_task_monitor)
                .is_none()
        );
        Self {
            // maybe don't use middleware but just add in here direcctly?
            router: self.router.route(
                path,
                match *method {
                    Method::GET => axum::routing::get(handler),
                    Method::POST => axum::routing::post(handler),
                    _ => unreachable!(),
                },
            ),
            task_monitors: self.task_monitors,
        }
    }

    pub fn finish(self) -> Router<MyState> {
        self.router.layer(TokioTaskMetricsLayer {
            task_monitors: self.task_monitors,
        })
    }
}
