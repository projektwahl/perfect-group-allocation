use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect};
use axum::TypedHeader;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, InsertResult};

use crate::entities::project_history::{self, ActiveModel};
use crate::error::AppErrorWithMetadata;
use crate::session::Session;
use crate::templating::render;
use crate::{CreateProject, CreateProjectPayload, CsrfSafeForm, XRequestId};

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn create(
    State(db): State<DatabaseConnection>,
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    session: Session,
    form: CsrfSafeForm<CreateProjectPayload>,
) -> Result<(Session, impl IntoResponse), AppErrorWithMetadata> {
    let result = async {
        let mut title_error = None;
        let mut description_error = None;

        if form.value.title.is_empty() {
            title_error = Some("title must not be empty".to_owned());
        }

        if form.value.description.is_empty() {
            description_error = Some("description must not be empty".to_owned());
        }

        if title_error.is_some() || description_error.is_some() {
            let result = render(
                &session,
                "create-project",
                CreateProject {
                    title: Some(form.value.title.clone()),
                    title_error,
                    description: Some(form.value.description.clone()),
                    description_error,
                },
            )
            .await?;
            return Ok(Html(result).into_response());
        }

        let project = project_history::ActiveModel {
            id: ActiveValue::Set(1_i32),
            title: ActiveValue::Set(form.value.title.clone()),
            description: ActiveValue::Set(form.value.description.clone()),
            ..Default::default()
        };
        let _unused: InsertResult<ActiveModel> =
            project_history::Entity::insert(project).exec(&db).await?;

        Ok(Redirect::to("/list").into_response())
    };
    match result.await {
        Ok(ok) => Ok((session, ok)),
        Err(app_error) => {
            // TODO FIXME store request id type-safe in body/session
            Err(AppErrorWithMetadata {
                session,
                request_id,
                app_error,
            })
        }
    }
}
