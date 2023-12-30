use std::collections::HashMap;

use axum::handler::Handler;
use axum::routing::MethodRouter;
use axum::Router;
use opentelemetry::metrics::Unit;
use opentelemetry::KeyValue;
use tokio_metrics::TaskMonitor;
use tracing::debug;

use crate::telemetry::tokio_metrics::TokioTaskMetricsLayer;
use crate::MyState;

#[derive(Default)]
pub struct MyRouter {
    router: Router<MyState>,
    task_monitors: HashMap<(Method, String), TaskMonitor>,
}

#[derive(Debug, PartialEq, Eq)]
enum Method {
    Get,
    Post,
}

impl MyRouter {
    pub fn new() -> Self {
        Self::default()
    }

    #[track_caller]
    #[must_use]
    pub fn route<T: 'static, H: Handler<T, MyState>>(
        mut self,
        path: &'static str,
        method: Method,
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
                    debug!("metrics for {}", path);
                    let task_metrics = interval_root.lock().unwrap().next().unwrap();
                    observer.observe_u64(
                        &mean_poll_duration,
                        task_metrics.mean_poll_duration().subsec_nanos().into(),
                        &[
                            KeyValue::new(
                                opentelemetry_semantic_conventions::trace::HTTP_REQUEST_METHOD,
                                path,
                            ),
                            KeyValue::new(
                                opentelemetry_semantic_conventions::trace::URL_PATH,
                                path,
                            ),
                        ],
                    );
                    observer.observe_f64(
                        &slow_poll_ratio,
                        task_metrics.slow_poll_ratio(),
                        &[KeyValue::new(
                            opentelemetry_semantic_conventions::trace::URL_PATH,
                            path,
                        )],
                    );
                },
            )
            .unwrap();
        assert!(
            self.task_monitors
                .insert(path.to_owned(), new_task_monitor)
                .is_none()
        );
        Self {
            router: self.router.route(path, method_router),
            task_monitors: self.task_monitors,
        }
    }

    pub fn finish(self) -> Router<MyState> {
        self.router.layer(TokioTaskMetricsLayer {
            task_monitors: self.task_monitors,
        })
    }
}
