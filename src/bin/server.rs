#![deny(warnings)]

#[tokio::main]
async fn main() {
    use warp::Filter;

    // Match any request and return hello world!
    let routes = warp::any().map(|| "Hello, World!");

    warp::serve(routes)
        .tls()
        .cert_path("example.com.crt")
        .key_path("example.com.key")
        .run(([127, 0, 0, 1], 3030))
        .await;
}
