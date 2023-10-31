use tokio::task::spawn_blocking;

use crate::error::AppError;
use crate::session::Session;
use crate::HANDLEBARS;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TemplateWrapper<'a, T> {
    pub csrf_token: &'a str,
    pub email: Option<&'a str>,
    #[serde(flatten)]
    pub inner: T,
}

pub async fn render<T: serde::Serialize + Send + 'static>(
    session: &Session,
    template_name: &'static str,
    value: T,
) -> Result<String, AppError> {
    let session = session.session();
    Ok(spawn_blocking(move || {
        HANDLEBARS
            .render(
                template_name,
                &TemplateWrapper {
                    csrf_token: &session.0,
                    email: session
                        .1
                        .as_ref()
                        .map(|session_cookie| session_cookie.email.as_str()),
                    inner: value,
                },
            )
            .unwrap_or_else(|render_error| render_error.to_string())
    })
    .await?)
}
