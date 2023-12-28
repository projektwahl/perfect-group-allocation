use axum::response::{Html, IntoResponse};
use axum_extra::TypedHeader;

use crate::error::to_error_result;
use crate::session::Session;
use crate::{CreateProject, XRequestId};

#[axum::debug_handler(state=crate::MyState)]
pub async fn index(
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    session: Session,
) -> Result<(Session, impl IntoResponse), (Session, impl IntoResponse)> {
    let result = async {
        let result = render(
            &session,
            "create-project",
            &CreateProject {
                title: None,
                title_error: None,
                description: None,
                description_error: None,
            },
        )
        .await;
        Ok(Html(result))
    };
    match result.await {
        Ok(ok) => Ok((session, ok)),
        Err(app_error) => Err(to_error_result(session, request_id, app_error).await),
    }
}
