use std::path::Path;
use std::sync::OnceLock;

use axum::response::IntoResponse;
use axum_extra::response::Css;
use axum_extra::{headers, TypedHeader};
use http::{header, StatusCode};
use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions};
use lightningcss::targets::Targets;
use parcel_sourcemap::SourceMap;

use crate::session::Session;

// add watcher and then use websocket to hot reload on client?
// or for dev simply enforce unbundled development where chrome directly modifies the files
// so maybe simply don't implement watcher at all

static INDEX_CSS: OnceLock<String> = OnceLock::new();

pub fn initialize_index_css() {
    // @import would produce a flash of unstyled content and also is less efficient
    let fs = FileProvider::new();
    let mut bundler = Bundler::new(&fs, None, ParserOptions::default());
    // TODO FIXME project independent path
    let stylesheet = bundler
        .bundle(Path::new("../frontend/index.css"))
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
        .unwrap();
}

// Etag and cache busting
pub async fn indexcss(
    if_none_match: TypedHeader<headers::IfNoneMatch>,
    session: Session,
) -> (Session, impl IntoResponse) {
    let etag_string = "\"xyzzy\"";
    let etag = etag_string.parse::<headers::ETag>().unwrap();
    if if_none_match.precondition_passes(&etag) {
        (
            session,
            (
                [
                    (header::ETAG, etag_string),
                    (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                ],
                Css(INDEX_CSS.get().unwrap().as_str()),
            )
                .into_response(),
        )
    } else {
        (session, StatusCode::NOT_MODIFIED.into_response())
    }
}
