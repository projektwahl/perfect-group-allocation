use std::collections::HashMap;

use axum::handler::Handler;
use axum::routing::MethodRouter;
use axum::Router;
use http::Method;
use opentelemetry::metrics::Unit;
use opentelemetry::KeyValue;
use tokio_metrics::TaskMonitor;
use tracing::debug;

use crate::telemetry::tokio_metrics::{BorrowedMethodAndPath, TokioTaskMetricsLayer};
use crate::MyState;

#[derive(Default)]
pub struct MyRouter {
    router: Router<MyState>,
    task_monitors: HashMap<BorrowedMethodAndPath<'static>, TaskMonitor>,
}

impl MyRouter {
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
        let new_task_monitor = TaskMonitor::default();

        let interval_root = std::sync::Mutex::new(new_task_monitor.intervals());

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
                    let task_metrics = interval_root.lock().unwrap().next().unwrap();
                    let attrs = &[
                        KeyValue::new(
                            opentelemetry_semantic_conventions::trace::HTTP_REQUEST_METHOD,
                            method.as_str(),
                        ),
                        KeyValue::new(opentelemetry_semantic_conventions::trace::URL_PATH, path),
                    ];
                    debug!(
                        "metrics for {} {} {:?}",
                        method,
                        path,
                        task_metrics.mean_poll_duration().subsec_nanos()
                    );
                    observer.observe_u64(
                        &mean_poll_duration,
                        task_metrics.mean_poll_duration().subsec_nanos().into(),
                        attrs,
                    );
                    observer.observe_f64(&slow_poll_ratio, task_metrics.slow_poll_ratio(), attrs);
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
                match method {
                    &Method::GET => axum::routing::get(handler),
                    &Method::POST => axum::routing::post(handler),
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
