use std::convert::Infallible;
use std::pin::pin;
use std::sync::Arc;

use async_zero_cost_templating::{html, TemplateToStream};
use bytes::Bytes;
use futures_util::StreamExt as _;
use http::{Response, StatusCode};
use http_body::Body;

use perfect_group_allocation_config::Config;

use crate::components::main::main;
use crate::error::AppError;

use crate::session::{ResponseSessionExt as _, Session};

pub async fn index(
    session: Session,
    config: &Config,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = Infallible> + Send + 'static>, AppError>
{
    let result = {
        let (tx_orig, rx) = tokio::sync::mpsc::channel(1);

        let tx = tx_orig.clone();

        let future = async move {
            html! {
                <h1 class="center">"Welcome"</h1>

                <p>"This is the starting page."</p>
            }
        };
        let future = main(tx_orig, "Home Page".into(), &session, &config, future);
        let stream = pin!(TemplateToStream::new(future, rx));
        // I think we should sent it at once with a content length when it is not too large
        stream.collect::<String>().await
    };

    Ok(Response::builder()
        .with_session(session)
        .status(StatusCode::OK)
        .body(result)
        .unwrap())
}
