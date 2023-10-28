use alloc::sync::Arc;

use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect};
use axum::TypedHeader;
use handlebars::Handlebars;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};

use crate::entities::project_history::{self};
use crate::error::AppErrorWithMetadata;
use crate::{CreateProject, CreateProjectPayload, CsrfSafeForm, ExtractSession, XRequestId};

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn create(
    State(db): State<DatabaseConnection>,
    State(handlebars): State<Arc<Handlebars<'static>>>,
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    ExtractSession {
        extractor: form,
        session,
    }: ExtractSession<CsrfSafeForm<CreateProjectPayload>>,
) -> Result<impl IntoResponse, AppErrorWithMetadata> {
    let mut session = session.lock().await;
    let expected_csrf_token = session.session_id();
    drop(session);
    let result = async {
        let mut title_error = None;
        let mut description_error = None;

        if form.value.title.is_empty() {
            title_error = Some("title must not be empty".to_string());
        }

        if form.value.description.is_empty() {
            description_error = Some("description must not be empty".to_string());
        }

        if title_error.is_some() || description_error.is_some() {
            let result = handlebars
                .render(
                    "create-project",
                    &CreateProject {
                        csrf_token: expected_csrf_token.clone(),
                        title: Some(form.value.title.clone()),
                        title_error,
                        description: Some(form.value.description.clone()),
                        description_error,
                    },
                )
                .unwrap_or_else(|e| e.to_string());
            return Ok(Html(result).into_response());
        }

        let project = project_history::ActiveModel {
            id: ActiveValue::Set(1),
            title: ActiveValue::Set(form.value.title.clone()),
            description: ActiveValue::Set(form.value.description.clone()),
            ..Default::default()
        };
        let _ = project_history::Entity::insert(project).exec(&db).await?;

        Ok(Redirect::to("/list").into_response())
    };
    match result.await {
        Ok(v) => Ok(v),
        Err(app_error) => {
            // TODO FIXME store request id type-safe in body/session
            Err(AppErrorWithMetadata {
                csrf_token: expected_csrf_token.clone(),
                request_id,
                handlebars,
                app_error,
            })
        }
    }
}
