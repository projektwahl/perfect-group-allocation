use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect};
use axum_extra::TypedHeader;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, InsertResult};
use zero_cost_templating::{template_stream, yieldoki, yieldokv};

use super::list::create_project;
use crate::entities::project_history::{self, ActiveModel};
use crate::error::to_error_result;
use crate::session::Session;
use crate::{CreateProject, CreateProjectPayload, CsrfSafeForm, XRequestId};

pub async fn create(
    State(db): State<DatabaseConnection>,
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    session: Session,
    form: CsrfSafeForm<CreateProjectPayload>,
) -> Result<(Session, impl IntoResponse), (Session, impl IntoResponse)> {
    let result = async gen {
        let template = yieldoki!(create_project());
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next());
        let template = yieldokv!(template.page_title("Create Project"));
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next_false());
        let template = yieldokv!(template.csrf_token("TODO"));
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next());
        let template = yieldoki!(template.next());
        let template = yieldokv!(template.csrf_token("TODO"));
        let template = yieldoki!(template.next());
        let template = yieldokv!(template.title(form.value.title));
        let template = yieldoki!(template.next());
        let has_errors = false;
        let template = if form.value.title.is_empty() {
            has_errors = true;
            let template = yieldoki!(template.next_true());
            let template = yieldokv!(template.title_error("title must not be empty"));
            yieldoki!(template.next())
        } else {
            yieldoki!(template.next_false())
        };
        let template = yieldokv!(template.description(form.value.description));
        let template = yieldoki!(template.next());
        let template = if form.value.description.is_empty() {
            has_errors = true;
            let template = yieldoki!(template.next_true());
            let template = yieldokv!(template.description_error("description must not be empty"));
            yieldoki!(template.next())
        } else {
            yieldoki!(template.next_false())
        };

        if has_errors {
            return;
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
        Err(app_error) => Err(to_error_result(session, request_id, app_error).await),
    }
}
