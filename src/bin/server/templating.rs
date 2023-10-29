use crate::csrf_protection::WithCsrfToken;
use crate::session::Session;
use crate::HANDLEBARS;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TemplateWrapper<'a, T> {
    pub csrf_token: &'a str,
    pub email: Option<&'a str>,
    #[serde(flatten)]
    pub inner: T,
}

pub fn render<T: serde::Serialize>(session: &mut Session, template_name: &str, value: T) {
    let session = session.session();
    HANDLEBARS
        .render(
            template_name,
            &TemplateWrapper {
                csrf_token: &session.0,
                email: session.1.as_ref().map(|s| s.email.as_str()),
                inner: value,
            },
        )
        .unwrap_or_else(|render_error| render_error.to_string());
}
