use std::path::PathBuf;

use selenium_manager::get_manager_by_browser;
use thirtyfour::prelude::*;
use tokio::process::Command;

fn main() -> WebDriverResult<()> {
    let mut manager = get_manager_by_browser("firefox".to_owned()).unwrap();
    let result = manager.setup().unwrap();
    println!("{:?}", result);

    // workaround because https://github.com/SeleniumHQ/selenium/blob/3f9b606c8444832df27425dc379ee092d52b42b2/rust/src/downloads.rs#L30 starts it's own runtime
    async_main(result)
}

#[tokio::main]
async fn async_main(path: PathBuf) -> WebDriverResult<()> {
    let mut command = Command::new(path).kill_on_drop(true).spawn().unwrap();

    let result = work().await;

    command.kill().await?;

    result
}

async fn work() -> WebDriverResult<()> {
    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    // Navigate to https://wikipedia.org.
    driver.goto("https://wikipedia.org").await?;
    let elem_form = driver.find(By::Id("search-form")).await?;

    // Find element from element.
    let elem_text = elem_form.find(By::Id("searchInput")).await?;

    // Type in the search terms.
    elem_text.send_keys("selenium").await?;

    // Click the search button.
    let elem_button = elem_form.find(By::Css("button[type='submit']")).await?;
    elem_button.click().await?;

    // Look for header to implicitly wait for the page to load.
    driver.find(By::ClassName("firstHeading")).await?;
    assert_eq!(driver.title().await?, "Selenium - Wikipedia");

    // Always explicitly close the browser.
    driver.quit().await?;

    Ok(())
}
