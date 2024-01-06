use futures::Future;
use tokio::sync::broadcast;

use crate::browsing_context::BrowsingContext;
use crate::webdriver::WebDriver;
use crate::webdriver_handler::SendCommand;
use crate::{browsing_context, log, session};

#[derive(Debug)]
pub struct WebDriverSession {
    pub session_id: String,
    pub driver: WebDriver,
}

impl WebDriverSession {
    pub fn session_end(
        &mut self,
    ) -> impl Future<Output = crate::result::Result<session::end::Result>> {
        self.driver.send_command(
            session::end::Command {
                params: session::end::Parameters {},
            },
            SendCommand::SessionEnd,
        )
    }

    pub fn browsing_context_get_tree(
        &mut self,
    ) -> impl Future<Output = crate::result::Result<browsing_context::get_tree::Result>> {
        self.driver.send_command(
            browsing_context::get_tree::Command {
                params: browsing_context::get_tree::Parameters {
                    max_depth: None,
                    root: None,
                },
            },
            SendCommand::BrowsingContextGetTree,
        )
    }

    pub fn browsing_context_navigate(
        &mut self,
        context: BrowsingContext,
        url: String,
    ) -> impl Future<Output = crate::result::Result<browsing_context::navigate::Result>> {
        self.driver.send_command(
            browsing_context::navigate::Command {
                params: browsing_context::navigate::Parameters {
                    context,
                    url,
                    wait: browsing_context::ReadinessState::Complete,
                },
            },
            SendCommand::BrowsingContextNavigate,
        )
    }

    pub fn subscribe_global_logs(
        &mut self,
    ) -> impl Future<Output = crate::result::Result<broadcast::Receiver<log::EntryAdded>>> {
        self.driver
            .request_subscribe(None, SendCommand::SubscribeGlobalLogs)
    }

    pub fn subscribe_logs(
        &mut self,
        browsing_context: BrowsingContext,
    ) -> impl Future<Output = crate::result::Result<broadcast::Receiver<log::EntryAdded>>> {
        self.driver
            .request_subscribe(Some(browsing_context), SendCommand::SubscribeGlobalLogs)
    }
}
