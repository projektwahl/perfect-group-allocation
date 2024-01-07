use paste::paste;

macro_rules! magic {
    (
        pub enum {
            $(#[doc = $doc:expr] $variant:ident($tag:literal $($command:ident)::+)),*
        }
        pub enum {
            $(#[doc = $doc_subscription:expr] $variant_subscription:ident($tag_subscription:literal $($command_subscription:ident)::+)),*
        }
    ) => {
        paste! {

            /// <https://w3c.github.io/webdriver-bidi/#protocol>
            #[derive(Debug)]
            pub enum SendCommand {
                $(#[doc = $doc] $variant($($command)::*::Command, ::tokio::sync::oneshot::Sender<$($command)::*::Result>),)*
                $(#[doc = $doc_subscription] $variant_subscription(Option<crate::BrowsingContext>, ::tokio::sync::oneshot::Sender<::tokio::sync::broadcast::Receiver<$($command_subscription)::*>>),)*
            }

            /// <https://w3c.github.io/webdriver-bidi/#protocol>
            #[derive(Debug)]
            pub enum RespondCommand {
                $(#[doc = $doc] $variant(::tokio::sync::oneshot::Sender<$($command)::*::Result>),)*
                $(#[doc = $doc_subscription] $variant_subscription(::tokio::sync::broadcast::Receiver<$($command_subscription)::*>, ::tokio::sync::oneshot::Sender<::tokio::sync::broadcast::Receiver<$($command_subscription)::*>>),)*
            }

            /// <https://w3c.github.io/webdriver-bidi/#protocol>
            #[derive(Debug, ::serde::Serialize, ::serde::Deserialize, Clone)]
            #[serde(tag = "method")]
            #[serde(rename_all = "camelCase")]
            #[serde(deny_unknown_fields)]
            pub enum CommandData {
                $(
                    #[doc = $doc]
                    #[serde(rename = $tag)]
                    $variant($($command)::*::Command)
                ,)*
            }

            /// <https://w3c.github.io/webdriver-bidi/#protocol>
            #[derive(Debug, ::serde::Serialize, ::serde::Deserialize, Clone)]
            #[serde(tag = "method")]
            #[serde(rename_all = "camelCase")]
            #[serde(deny_unknown_fields)]
            pub enum EventData {
                $(
                    #[doc = $doc_subscription]
                    #[serde(rename = $tag_subscription)]
                    $variant_subscription($($command_subscription)::*)
                ,)*
            }

            impl crate::ExtractBrowsingContext for EventData {
                fn browsing_context(&self) -> Option<&crate::browsing_context::BrowsingContext> {
                    match self {
                        $(
                            EventData::$variant_subscription(event) => {
                                event.browsing_context()
                            }
                        ),*
                    }
                }
            }

            /// <https://w3c.github.io/webdriver-bidi/#protocol>
            #[derive(Debug, Default)]
            pub struct GlobalEventSubscription {
                $(
                    #[doc = $doc_subscription]
                    [<$variant_subscription:snake>]: Option<(::tokio::sync::broadcast::Sender<$($command_subscription)::*>, ::tokio::sync::broadcast::Receiver<$($command_subscription)::*>)>
                ,)*
            }

            /// <https://w3c.github.io/webdriver-bidi/#protocol>
            #[derive(Debug, Default)]
            pub struct EventSubscription {
                $(
                    #[doc = $doc_subscription]
                    [<$variant_subscription:snake>]:
                        ::std::collections::HashMap<crate::BrowsingContext, (::tokio::sync::broadcast::Sender<$($command_subscription)::*>, ::tokio::sync::broadcast::Receiver<$($command_subscription)::*>)>
                ,)*
            }


            /// <https://w3c.github.io/webdriver-bidi/#protocol>
            #[derive(Debug, ::serde::Serialize, ::serde::Deserialize, Clone)]
            #[serde(tag = "method")]
            #[serde(rename_all = "camelCase")]
            #[serde(deny_unknown_fields)]
            pub enum ResultData {
                $(
                    #[doc = $doc]
                    #[serde(rename = $tag)]
                    $variant($($command)::*::Result)
                ,)*
            }

            pub(crate) async fn handle_command(this: &mut crate::webdriver_handler::WebDriverHandler, input: SendCommand) -> crate::result::Result<()> {
                match input {
                    $(
                        SendCommand::$variant(command, sender) => {
                            this.handle_command_internal(command, sender, RespondCommand::$variant).await?;
                        }
                    ),*
                    $(
                        SendCommand::$variant_subscription(command, sender) => {
                            match command {
                                Some(browsing_context) => {
                                    this.handle_subscription_internal($tag_subscription.to_owned(), browsing_context, sender, |ges| &mut ges.[<$variant_subscription:snake>], RespondCommand::$variant_subscription).await?;
                                }
                                None => {
                                    this.handle_global_subscription_internal($tag_subscription.to_owned(), sender, |ges| &mut ges.[<$variant_subscription:snake>], RespondCommand::$variant_subscription).await?;
                                }
                            }
                        }
                    ),*
                }
                Ok(())
            }

            pub(crate) fn handle_event(this: &mut crate::webdriver_handler::WebDriverHandler, input: EventData) -> crate::result::Result<()> {
                match input {
                    $(
                        EventData::$variant_subscription(event) => {
                            // TODO FIXME extract method

                            // maybe no global but only browsercontext subscription
                            if let Some(sub) = this.global_subscriptions.[<$variant_subscription:snake>].as_ref() {
                                // TODO FIXME don't unwrap but unsubscribe in this case
                                sub.0.send(event.clone()).unwrap();
                            }
                            // we should find out in which cases there is no browsing context
                            if let Some(browsing_context) = <$($command_subscription)::* as crate::ExtractBrowsingContext>::browsing_context(&event) {
                                // maybe global but no browsercontext subscription
                                if let Some(sub) = this.subscriptions.[<$variant_subscription:snake>].get(browsing_context) {
                                    // TODO FIXME don't unwrap but unsubscribe in this case
                                    sub.0.send(event).unwrap();
                                }
                            }
                        }
                    ),*
                }
                Ok(())
            }

            pub(crate) fn send_response(_this: &mut crate::webdriver_handler::WebDriverHandler, result: ::serde_json::Value, respond_command: RespondCommand) -> crate::result::Result<()> {
                match (respond_command) {
                    $(
                        RespondCommand::$variant(respond_command) => {
                            respond_command
                                .send(serde_path_to_error::deserialize(result)
                                    .map_err(crate::result::ErrorInner::ParseReceivedWithPath)?)
                                .map_err(|_| crate::result::ErrorInner::CommandCallerExited)?
                        }
                    ),*
                    $(
                        RespondCommand::$variant_subscription(value, channel) => {
                            // result here is the result of the subscribe command which should be empty
                            // serde_path_to_error::deserialize(result).map_err(crate::result::Error::ParseReceivedWithPath)?

                            // TODO FIXME we need to know whether this was a global or local subscription. maybe store that directly in the respond command?
                            channel.send(value)
                                .map_err(|_| crate::result::ErrorInner::CommandCallerExited)?
                        }
                    ),*
                }
                Ok(())
            }
        }
    };
}

magic! {
    pub enum {
        /// <https://w3c.github.io/webdriver-bidi/#command-session-new>
        SessionNew("session.new" crate::session::new),
        /// <https://w3c.github.io/webdriver-bidi/#command-session-end>
        SessionEnd("session.end" crate::session::end),
        /// <https://w3c.github.io/webdriver-bidi/#command-session-subscribe>
        SessionSubscribe("session.subscribe" crate::session::subscribe), // TODO FIXME this should not be in sendcommand
        /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
        BrowsingContextGetTree("browsingContext.getTree" crate::browsing_context::get_tree),
        /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
        BrowsingContextNavigate("browsingContext.navigate" crate::browsing_context::navigate),
        /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-create>
        BrowsingContextCreate("browsingContext.create" crate::browsing_context::create)
    }
    pub enum {
        /// tmp
        SubscribeGlobalLogs("log.entryAdded" crate::log::EntryAdded)
    }
}
