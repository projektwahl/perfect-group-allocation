use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect};
use handlebars::Handlebars;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};

use crate::entities::project_history::{self, Entity};
use crate::error::AppError;
use crate::{CreateProject, CreateProjectPayload, CsrfSafeForm, ExtractSession, MyBody, MyState};

#[axum::debug_handler(body=MyBody, state=MyState)]
pub async fn create(
    State(db): State<DatabaseConnection>,
    State(handlebars): State<Handlebars<'static>>,
    ExtractSession {
        extractor: form,
        session,
    }: ExtractSession<CsrfSafeForm<CreateProjectPayload>>,
) -> Result<impl IntoResponse, AppError> {
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
                    csrf_token: session.lock().await.session_id(),
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
}
