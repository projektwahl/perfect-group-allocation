#[try_stream(ok = String, error = DbErr)]
async fn list_internal(db: DatabaseConnection, handlebars: Handlebars<'static>) {
    let stream = ProjectHistory::find().stream(&db).await.unwrap();
    yield handlebars
        .render("main_pre", &json!({"page_title": "Projects"}))
        .unwrap_or_else(|e| e.to_string());
    #[for_await]
    for x in stream {
        let x = x?;
        let result = handlebars
            .render(
                "project",
                &TemplateProject {
                    title: x.title,
                    description: x.description,
                },
            )
            .unwrap_or_else(|e| e.to_string());
        yield result;
    }
    yield handlebars
        .render("main_post", &json!({}))
        .unwrap_or_else(|e| e.to_string());
}

#[axum::debug_handler(body=MyBody, state=MyState)]
async fn list(
    State(db): State<DatabaseConnection>,
    State(handlebars): State<Handlebars<'static>>,
) -> impl IntoResponse {
    let stream = list_internal(db, handlebars).map(|elem| match elem {
        Err(v) => Ok(format!("<h1>Error {}</h1>", encode_safe(&v.to_string()))),
        o => o,
    });
    (
        [(header::CONTENT_TYPE, "text/html")],
        StreamBody::new(stream),
    )
}
