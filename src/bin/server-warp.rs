#![deny(warnings)]

use http::Response;

#[tokio::main]
async fn main() {
    use warp::Filter;

    // Match any request and return hello world!
    let routes = warp::any().map(|| {
        Response::builder()
            .header("Alt-Svc", r#"h3=":443"; ma=2592000"#) // needs to be < 1024
            .body("and a custom body")
    });

    // warp doesn't support ec keys
    warp::serve(routes)
        .tls()
        .cert_path(".lego/certificates/h3.selfmade4u.de.crt")
        .key_path(".lego/certificates/h3.selfmade4u.de.key")
        .run(([127, 0, 0, 1], 443))
        .await;
}
