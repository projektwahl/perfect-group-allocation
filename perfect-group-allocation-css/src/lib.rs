use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions};
use lightningcss::targets::Targets;
use proc_macro::{quote, TokenStream, TokenTree};

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
    // let mut source_map = SourceMap::new(".");

    let result = stylesheet
        .to_css(PrinterOptions {
            minify: true,
            source_map: None, //Some(&mut source_map),
            project_root: None,
            targets: Targets::default(),
            analyze_dependencies: None,
            pseudo_classes: None,
        })
        .unwrap()
        .code;
    let mut hasher = DefaultHasher::new();
    result.hash(&mut hasher);
    let hash = TokenTree::Literal(proc_macro::Literal::u64_suffixed(hasher.finish()));
    let result = TokenTree::Literal(proc_macro::Literal::byte_string(result.as_bytes()));
    quote! {
        ($result, $hash)
    }
}
