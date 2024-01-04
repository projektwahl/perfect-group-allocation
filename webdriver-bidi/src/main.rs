use webdriver_bidi::WebDriverBiDi;

#[tokio::main]
pub async fn main() -> Result<(), tokio_tungstenite::tungstenite::Error> {
    let mut driver = WebDriverBiDi::new().await?;
    let session = driver.create_session().await?;

    Ok(())
}
