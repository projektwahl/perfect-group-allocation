use crate::webdriver::WebDriver;
use crate::{browsing_context, session, WebDriverBiDiRemoteEndCommandData};

#[derive(Debug)]
pub struct WebDriverSession {
    pub session_id: String,
    pub driver: WebDriver,
}

impl WebDriverSession {
    pub async fn session_end(mut self) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        let result: session::end::Result = self
            .driver
            .send_command(WebDriverBiDiRemoteEndCommandData::SessionCommand(
                session::Command::SessionEnd(session::end::CommandType {
                    params: session::end::Parameters {},
                }),
            ))
            .await?;
        println!("{result:?}");
        Ok(())
    }

    pub async fn browsing_context_get_tree(
        &mut self,
    ) -> Result<(), tokio_tungstenite::tungstenite::Error> {
        self.driver
            .send_command(WebDriverBiDiRemoteEndCommandData::BrowsingContext(
                browsing_context::Command::GetTree(browsing_context::get_tree::CommandType {
                    params: browsing_context::get_tree::Parameters {
                        max_depth: todo!(),
                        root: todo!(),
                    },
                }),
            ))
            .await
    }
}
