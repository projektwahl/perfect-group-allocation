use std::path::Path;
use std::sync::{Once, OnceLock};

use axum::response::IntoResponse;
use axum_extra::response::Css;
use axum_extra::TypedHeader;
use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions};
use lightningcss::targets::Targets;
use parcel_sourcemap::SourceMap;
use tokio::task::spawn_blocking;

use crate::error::to_error_result;
use crate::session::Session;
use crate::XRequestId;

// add watcher and then use websocket to hot reload on client?
// or for dev simply enforce unbundled development where chrome directly modifies the files
// so maybe simply don't implement watcher at all

static INDEX_CSS: OnceLock<String> = OnceLock::new();

pub fn initialize_index_css() {
    // @import would produce a flash of unstyled content and also is less efficient
    let fs = FileProvider::new();
    let mut bundler = Bundler::new(&fs, None, ParserOptions::default());
    let stylesheet = bundler
        .bundle(Path::new("frontend/index.css"))
        .map_err(|error| lightningcss::error::Error {
            kind: error.kind.to_string(),
            loc: error.loc,
        })
        .unwrap();
    let mut source_map = SourceMap::new(".");

    INDEX_CSS
        .set(
            stylesheet
                .to_css(PrinterOptions {
                    minify: true,
                    source_map: Some(&mut source_map),
                    project_root: None,
                    targets: Targets::default(),
                    analyze_dependencies: None,
                    pseudo_classes: None,
                })
                .unwrap()
                .code,
        )
        .unwrap()
}

#[axum::debug_handler(state=crate::MyState)]
pub async fn indexcss(
    TypedHeader(XRequestId(request_id)): TypedHeader<XRequestId>,
    session: Session,
) -> (Session, impl IntoResponse) {
    (session, Css(INDEX_CSS.get().unwrap().as_str()))
}
