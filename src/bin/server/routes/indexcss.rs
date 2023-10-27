#[axum::debug_handler(body=MyBody, state=MyState)]
async fn indexcss() -> impl IntoResponse {
    // @import would produce a flash of unstyled content and also is less efficient
    let fs = FileProvider::new();
    let mut bundler = Bundler::new(&fs, None, ParserOptions::default());
    let stylesheet = bundler.bundle(Path::new("frontend/index.css")).unwrap();
    let mut source_map = SourceMap::new(".");
    Css(stylesheet
        .to_css(PrinterOptions {
            minify: true,
            source_map: Some(&mut source_map),
            project_root: None,
            targets: Targets::default(),
            analyze_dependencies: None,
            pseudo_classes: None,
        })
        .unwrap()
        .code)
}
