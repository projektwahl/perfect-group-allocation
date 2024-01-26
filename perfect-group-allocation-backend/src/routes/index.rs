use std::convert::Infallible;

use bytes::Bytes;
use http::{Response, StatusCode};
use http_body::Body;
use http_body_util::StreamBody;
use perfect_group_allocation_config::Config;
use perfect_group_allocation_openidconnect::id_token_claims;

use crate::error::AppError;
use crate::routes::indexcss::INDEX_CSS_VERSION;
use crate::session::{ResponseSessionExt as _, Session};
use crate::{yieldfi, yieldfv};

pub async fn index(
    session: Session,
    config: Config,
) -> Result<hyper::Response<impl Body<Data = Bytes, Error = Infallible> + Send + 'static>, AppError>
{
    let csrf_token = session.csrf_token();
    let openidconnect_session = session.openidconnect_session();
    let result = async move {
        let template = yieldfi!(create_project());
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next());
        let template = yieldfv!(template.page_title("Create Project"));
        let template = yieldfi!(template.next());
        let template =
            yieldfv!(template
                .indexcss_version_unsafe(Unsafe::unsafe_input(INDEX_CSS_VERSION.to_string())));
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next());
        let template = if let Some(openidconnect_session) = openidconnect_session {
            let claims = id_token_claims(config, openidconnect_session)
                .await
                .unwrap();
            println!("{claims:?}");
            let template = yieldfi!(template.next_email_true());
            let template = yieldfv!(template.csrf_token(csrf_token.clone()));
            let template = yieldfi!(template.next());
            let template = yieldfv!(template.email(claims.email().unwrap().to_string()));
            yieldfi!(template.next())
        } else {
            let template = yieldfi!(template.next_email_false());
            let template = yieldfv!(template.csrf_token(csrf_token.clone()));
            yieldfi!(template.next())
        };
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next_error_false());
        let template = yieldfv!(template.csrf_token(csrf_token));
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next_title_error_false());
        let template = yieldfv!(template.title(""));
        let template = yieldfi!(template.next());
        let template = yieldfi!(template.next_description_error_false());
        let template = yieldfv!(template.description(""));
        let template = yieldfi!(template.next());
        yieldfi!(template.next());
    };
    let stream = AsyncIteratorStream(result);
    Ok(Response::builder()
        .with_session(session)
        .status(StatusCode::OK)
        .body(StreamBody::new(stream))
        .unwrap())
}
