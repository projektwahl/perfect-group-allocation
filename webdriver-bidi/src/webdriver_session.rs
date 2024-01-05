use crate::browsing_context::BrowsingContext;
use crate::session::SubscriptionRequest;
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
    ) -> Result<browsing_context::get_tree::Result, tokio_tungstenite::tungstenite::Error> {
        self.driver
            .send_command(WebDriverBiDiRemoteEndCommandData::BrowsingContext(
                browsing_context::Command::GetTree(browsing_context::get_tree::CommandType {
                    params: browsing_context::get_tree::Parameters {
                        max_depth: None,
                        root: None,
                    },
                }),
            ))
            .await
    }

    pub async fn browsing_context_navigate(
        &mut self,
        context: BrowsingContext,
        url: String,
    ) -> Result<browsing_context::navigate::Result, tokio_tungstenite::tungstenite::Error> {
        self.driver
            .send_command(WebDriverBiDiRemoteEndCommandData::BrowsingContext(
                browsing_context::Command::Navigate(browsing_context::navigate::CommandType {
                    params: browsing_context::navigate::Parameters {
                        context,
                        url,
                        wait: browsing_context::ReadinessState::Complete,
                    },
                }),
            ))
            .await
    }

    pub async fn session_subscribe(
        &mut self,
        browsing_context: BrowsingContext,
    ) -> Result<session::subscribe::Result, tokio_tungstenite::tungstenite::Error> {
        self.driver
            .send_command(WebDriverBiDiRemoteEndCommandData::SessionCommand(
                session::Command::Subscribe(session::subscribe::CommandType {
                    params: SubscriptionRequest {
                        events: vec!["log.entryAdded".to_owned()],
                        contexts: vec![browsing_context],
                    },
                }),
            ))
            .await
    }
}
