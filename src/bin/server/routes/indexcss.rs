use std::path::Path;

use axum::response::IntoResponse;
use axum::TypedHeader;
use axum_extra::response::Css;
use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions};
use lightningcss::targets::Targets;
use parcel_sourcemap::SourceMap;

use crate::error::AppErrorWithMetadata;
use crate::session::Session;
use crate::XRequestId;

#[axum::debug_handler(body=crate::MyBody, state=crate::MyState)]
pub async fn indexcss(
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    session: Session,
) -> Result<(Session, impl IntoResponse), AppErrorWithMetadata> {
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
