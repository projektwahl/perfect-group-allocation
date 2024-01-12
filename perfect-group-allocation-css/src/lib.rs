use std::path::Path;

use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions};
use lightningcss::targets::Targets;
use parcel_sourcemap::SourceMap;
use proc_macro::{TokenStream, TokenTree};

// TODO FIXME automatic recompilation

#[proc_macro]
pub fn index_css(_item: TokenStream) -> TokenStream {
    // @import would produce a flash of unstyled content and also is less efficient
    let fs = FileProvider::new();
    let mut bundler = Bundler::new(&fs, None, ParserOptions::default());
    // TODO FIXME project independent path
    let stylesheet = bundler
        .bundle(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../frontend/index.css"))
        .map_err(|error| lightningcss::error::Error {
            kind: error.kind.to_string(),
            loc: error.loc,
        })
        .unwrap();
    let mut source_map = SourceMap::new(".");

    let result = stylesheet
        .to_css(PrinterOptions {
            minify: true,
            source_map: Some(&mut source_map),
            project_root: None,
            targets: Targets::default(),
            analyze_dependencies: None,
            pseudo_classes: None,
        })
        .unwrap()
        .code;
    let tree = TokenTree::Literal(proc_macro::Literal::byte_string(result.as_bytes()));
    tree.into()
}
