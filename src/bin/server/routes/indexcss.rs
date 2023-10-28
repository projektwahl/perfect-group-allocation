use alloc::sync::Arc;
use std::path::Path;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::TypedHeader;
use axum_extra::response::Css;
use futures_util::TryFutureExt;
use handlebars::Handlebars;
use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions};
use lightningcss::targets::Targets;
use parcel_sourcemap::SourceMap;

use crate::error::AppErrorWithMetadata;
use crate::{EmptyBody, ExtractSession, XRequestId};

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn indexcss(
    State(handlebars): State<Arc<Handlebars<'static>>>,
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    ExtractSession { session, .. }: ExtractSession<EmptyBody>,
) -> Result<impl IntoResponse, AppErrorWithMetadata> {
    let mut session = session.lock().await;
    let expected_csrf_token = session.session_id();
    drop(session);
    let result = async {
        // @import would produce a flash of unstyled content and also is less efficient
        let fs = FileProvider::new();
        let mut bundler = Bundler::new(&fs, None, ParserOptions::default());
        let stylesheet = bundler
            .bundle(Path::new("frontend/index.css"))
            .map_err(|error| lightningcss::error::Error {
                kind: error.kind.to_string(),
                loc: error.loc,
            })?;
        let mut source_map = SourceMap::new(".");
        Ok(Css(stylesheet
            .to_css(PrinterOptions {
                minify: true,
                source_map: Some(&mut source_map),
                project_root: None,
                targets: Targets::default(),
                analyze_dependencies: None,
                pseudo_classes: None,
            })?
            .code))
    };
    result
        .map_err(|app_error| {
            // TODO FIXME store request id type-safe in body/session
            AppErrorWithMetadata {
                csrf_token: expected_csrf_token.clone(),
                request_id,
                handlebars,
                app_error,
            }
        })
        .await
}
