use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::task::{ready, Context, Poll};

use axum::extract::MatchedPath;
use http::{HeaderMap, HeaderName, HeaderValue, Request, Response};
use opentelemetry::global;
use opentelemetry::propagation::{Extractor, Injector};
use opentelemetry_semantic_conventions::trace as otelsc;
use pin_project::pin_project;
use tower::{Layer, Service};
use tracing::instrument::Instrumented;
use tracing::Instrument as _;
use tracing_opentelemetry::OpenTelemetrySpanExt;

// TODO FIXME add support for http client

// inspired by https://github.com/tower-rs/tower-http/blob/main/tower-http/src/trace/service.rs

#[derive(Debug, Clone)]
pub struct MyTraceLayer;

impl<S> Layer<S> for MyTraceLayer {
    type Service = MyTraceService<S>;

    fn layer(&self, service: S) -> Self::Service {
        MyTraceService { inner: service }
    }
}

#[derive(Debug, Clone)]
pub struct MyTraceService<S> {
    inner: S,
}

impl<S, RequestBody, ResponseBody> Service<Request<RequestBody>> for MyTraceService<S>
where
    S: Service<Request<RequestBody>, Response = Response<ResponseBody>, Error = Infallible>,
{
    type Error = Infallible;
    type Future = MyTraceFuture<S::Future>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<RequestBody>) -> Self::Future {
        let _context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&MyTraceHeaderPropagator(request.headers_mut()))
        });

        // https://opentelemetry.io/docs/specs/semconv/attributes-registry/http/
        // https://opentelemetry.io/docs/specs/semconv/http/http-spans/

        let route = request
            .extensions()
            .get::<MatchedPath>()
            .unwrap()
            .as_str()
            .to_owned();

        let span = tracing::span!(
            tracing::Level::DEBUG,
            "request",
            "otel.name" = format!("{} {}", request.method(), route),
        );

        // TODO FIXME set parent as the connection if no other parent is provided. Or maybe at least somehow link them.
        //span.set_parent(context);

        if let Some(value) = request
            .headers()
            .get(http::header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
        {
            span.set_attribute(otelsc::USER_AGENT_ORIGINAL, value.to_owned());
        }

        span.set_attribute(otelsc::URL_FULL, request.uri().to_string());
        span.set_attribute(otelsc::URL_PATH, request.uri().path().to_string());
        if let Some(query) = request.uri().query() {
            span.set_attribute(otelsc::URL_QUERY, query.to_string());
        }
        if let Some(scheme) = request.uri().scheme() {
            span.set_attribute(otelsc::URL_SCHEME, scheme.to_string());
        }
        for (name, value) in request.headers() {
            if let Ok(value) = value.to_str() {
                // TODO FIXME escaping
                span.set_attribute(format!("http.request.header.{name}"), value.to_owned());
            }
        }
        span.set_attribute(otelsc::HTTP_REQUEST_METHOD, request.method().to_string());

        span.set_attribute(otelsc::HTTP_ROUTE, route);

        let response_future = {
            let _ = span.enter();
            let uninstrumented_future = self.inner.call(request);
            uninstrumented_future.instrument(span)
        };

        MyTraceFuture { response_future }
    }
}

#[pin_project]
pub struct MyTraceFuture<F> {
    #[pin]
    response_future: Instrumented<F>,
}

impl<F, ResponseBody> Future for MyTraceFuture<F>
where
    F: Future<Output = Result<Response<ResponseBody>, Infallible>>,
{
    type Output = Result<Response<ResponseBody>, Infallible>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        let poll_result = ready!(this.response_future.as_mut().poll(cx));
        let span = this.response_future.span();

        match poll_result {
            Ok(mut response) => {
                // trace response
                global::get_text_map_propagator(|propagator| {
                    propagator.inject_context(
                        &this.response_future.span().context(),
                        // Can't use tower_http::trace::TraceLayer because we need mutable access to the response
                        &mut MyTraceHeaderPropagator(response.headers_mut()),
                    );
                });

                //span.set_attribute(otelsc::HTTP_RESPONSE_BODY_SIZE, response.)

                for (name, value) in response.headers() {
                    if let Ok(value) = value.to_str() {
                        // TODO FIXME escaping
                        span.set_attribute(
                            format!("http.response.header.{name}"),
                            value.to_owned(),
                        );
                    }
                }

                span.set_attribute(
                    otelsc::HTTP_RESPONSE_STATUS_CODE,
                    response.status().to_string(),
                );

                Poll::Ready(Ok(response))
            }
            // TODO FIXME maybe add catch panic handling, also services could explicitly return an internal server error without panicking which should also be traced
            Err(error) => match error {
                /*// error.type
                // https://opentelemetry.io/docs/specs/otel/trace/exceptions/
                // https://github.com/open-telemetry/semantic-conventions/blob/main/docs/exceptions/exceptions-spans.md
                error!(error = error, "test",);

                Poll::Ready(Err(error))*/
            },
        }
    }
}

// https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/#special-fields

pub struct MyTraceHeaderPropagator<'a>(&'a mut HeaderMap<HeaderValue>);

impl<'a> Injector for MyTraceHeaderPropagator<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = HeaderName::from_str(key) {
            if let Ok(value) = HeaderValue::from_str(&value) {
                self.0.insert(key, value);
            }
        }
    }
}

impl<'a> Extractor for MyTraceHeaderPropagator<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(http::HeaderName::as_str).collect()
    }
}
