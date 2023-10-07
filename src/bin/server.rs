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

    warp::serve(routes)
        .tls()
        .cert_path("example.com.cert.pem")
        .key_path("example.com.key.pem")
        .run(([127, 0, 0, 1], 443))
        .await;
}
