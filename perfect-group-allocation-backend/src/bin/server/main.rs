use perfect_group_allocation_backend::run_server;

pub fn main() {
    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let database_url = std::env::var("DATABASE_URL")?;
            run_server(database_url).await?.await
        });
    if let Err(err) = result {
        panic!("{err}")
    }
}
