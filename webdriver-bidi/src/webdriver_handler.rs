use std::collections::HashMap;
use std::fmt::Debug;

use futures::{SinkExt as _, StreamExt as _};
use paste::paste;
use serde::Serialize;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::trace;

use crate::browsing_context::BrowsingContext;
use crate::{
    browsing_context, log, session, WebDriverBiDiLocalEndCommandResponse,
    WebDriverBiDiLocalEndMessage, WebDriverBiDiLocalEndMessageErrorResponse,
    WebDriverBiDiRemoteEndCommand,
};

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

            /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
            #[derive(Debug)]
            pub enum SendCommand {
                $(#[doc = $doc] $variant($($command)::*::Command, oneshot::Sender<$($command)::*::Result>),)*
                $(#[doc = $doc_subscription] $variant_subscription(Option<BrowsingContext>, oneshot::Sender<broadcast::Receiver<$($command_subscription)::*>>),)*
            }

            /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
            #[derive(Debug)]
            pub enum RespondCommand {
                $(#[doc = $doc] $variant(oneshot::Sender<$($command)::*::Result>),)*
                $(#[doc = $doc_subscription] $variant_subscription(broadcast::Receiver<$($command_subscription)::*>, oneshot::Sender<broadcast::Receiver<$($command_subscription)::*>>),)*
            }

            /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
            #[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
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

            /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
            #[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
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

            /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
            #[derive(Debug, Default)]
            pub struct GlobalEventSubscription {
                $(
                    #[doc = $doc_subscription]
                    [<$variant_subscription:snake>]: Option<(broadcast::Sender<$($command_subscription)::*>, broadcast::Receiver<$($command_subscription)::*>)>
                ,)*
            }

            /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
            #[derive(Debug, Default)]
            pub struct EventSubscription {
                $(
                    #[doc = $doc_subscription]
                    [<$variant_subscription:snake>]:
                        HashMap<BrowsingContext, (broadcast::Sender<$($command_subscription)::*>, broadcast::Receiver<$($command_subscription)::*>)>
                ,)*
            }


            /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
            #[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
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

            async fn handle_command(this: &mut WebDriverHandler, input: SendCommand) -> crate::result::Result<()> {
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

            fn handle_event(this: &mut WebDriverHandler, input: EventData) -> crate::result::Result<()> {
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

            fn send_response(_this: &mut WebDriverHandler, result: Value, respond_command: RespondCommand) -> crate::result::Result<()> {
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
        SessionNew("session.new" session::new),
        /// <https://w3c.github.io/webdriver-bidi/#command-session-end>
        SessionEnd("session.end" session::end),
        /// <https://w3c.github.io/webdriver-bidi/#command-session-subscribe>
        SessionSubscribe("session.subscribe" session::subscribe), // TODO FIXME this should not be in sendcommand
        /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
        BrowsingContextGetTree("browsingContext.getTree" browsing_context::get_tree),
        /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
        BrowsingContextNavigate("browsingContext.navigate" browsing_context::navigate),
        /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-create>
        BrowsingContextCreate("browsingContext.create" browsing_context::create)
    }
    pub enum {
        /// tmp
        SubscribeGlobalLogs("log.entryAdded" log::EntryAdded)
    }
}

pub struct WebDriverHandler {
    id: u64,
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    receive_command: mpsc::UnboundedReceiver<SendCommand>,
    pending_commands: HashMap<u64, RespondCommand>,
    subscriptions: EventSubscription,
    global_subscriptions: GlobalEventSubscription,
}

impl WebDriverHandler {
    pub async fn handle(
        stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        receive_command: mpsc::UnboundedReceiver<SendCommand>,
    ) {
        let mut this = Self {
            id: 0,
            stream,
            receive_command,
            pending_commands: HashMap::default(),
            subscriptions: EventSubscription::default(),
            global_subscriptions: GlobalEventSubscription::default(),
        };
        this.handle_internal().await;
    }

    async fn handle_internal(&mut self) {
        loop {
            tokio::select! {
                // TODO FIXME make this truly parallel. e.g. if receiving a message while sending hangs
                message = self.stream.next() => {
                    match message {
                        Some(Ok(Message::Text(message))) => {
                            trace!("received {message}");
                            if let Err(error) = self.handle_message(&message) {
                                eprintln!("error when parsing incoming message {message} {error}");
                            }
                        }
                        Some(Ok(message)) => {
                            println!("Unknown message: {message}");
                        }
                        Some(Err(error)) => println!("Error in receive {error}"),
                        None => {
                            println!("connection closed");
                            break;
                        }
                    }
                }
                // TODO FIXME use the receive many functions
                Some(receive_command) = self.receive_command.recv() => {
                    if let Err(error) = handle_command(self, receive_command).await {
                        eprintln!("error when handling incoming command {error}");
                    }
                }
            }
        }
        println!("handle closed");
    }

    async fn handle_global_subscription_internal<R: Clone + Send>(
        &mut self,
        event: String,
        sender: oneshot::Sender<broadcast::Receiver<R>>,
        global_event_subscription: impl Fn(
            &mut GlobalEventSubscription,
        ) -> &mut Option<(
            broadcast::Sender<R>,
            broadcast::Receiver<R>,
        )> + Send,
        respond_command_constructor: impl FnOnce(
            broadcast::Receiver<R>,
            oneshot::Sender<broadcast::Receiver<R>>,
        ) -> RespondCommand
        + Send,
    ) -> crate::result::Result<()> {
        match global_event_subscription(&mut self.global_subscriptions) {
            Some(subscription) => {
                sender
                    .send(subscription.0.subscribe())
                    .map_err(|_| crate::result::ErrorInner::CommandCallerExited)?;
            }
            None => {
                self.id += 1;

                let ch = broadcast::channel(10);

                self.pending_commands.insert(
                    self.id,
                    respond_command_constructor(ch.0.subscribe(), sender),
                );

                let string = serde_json::to_string(&WebDriverBiDiRemoteEndCommand {
                    id: self.id,
                    command_data: session::subscribe::Command {
                        params: session::SubscriptionRequest {
                            events: vec![event],
                            contexts: vec![],
                        },
                    },
                })
                .unwrap();

                *global_event_subscription(&mut self.global_subscriptions) = Some(ch);

                trace!("sent {string}");

                // starting from here this could be done asynchronously
                // TODO FIXME I don't think we need the flushing requirement here specifically. maybe flush if no channel is ready or something like that
                self.stream
                    .send(Message::Text(string))
                    .await
                    .map_err(crate::result::ErrorInner::WebSocket)?;
            }
        };
        Ok(())
    }

    async fn handle_subscription_internal<R: Clone + Send>(
        &mut self,
        event: String,
        command_data: BrowsingContext,
        sender: oneshot::Sender<broadcast::Receiver<R>>,
        event_subscription: impl Fn(
            &mut EventSubscription,
        ) -> &mut HashMap<
            BrowsingContext,
            (broadcast::Sender<R>, broadcast::Receiver<R>),
        > + Send,
        respond_command_constructor: impl FnOnce(
            broadcast::Receiver<R>,
            oneshot::Sender<broadcast::Receiver<R>>,
        ) -> RespondCommand
        + Send,
    ) -> crate::result::Result<()> {
        if let Some(subscription) = event_subscription(&mut self.subscriptions).get(&command_data) {
            sender
                .send(subscription.0.subscribe())
                .map_err(|_| crate::result::ErrorInner::CommandCallerExited)?; // TODO FIXME this would return before the request command is actually done
        } else {
            self.id += 1;

            let ch = broadcast::channel(10);

            self.pending_commands.insert(
                self.id,
                respond_command_constructor(ch.0.subscribe(), sender),
            );

            let string = serde_json::to_string(&WebDriverBiDiRemoteEndCommand {
                id: self.id,
                command_data: session::subscribe::Command {
                    params: session::SubscriptionRequest {
                        events: vec![event],
                        contexts: vec![command_data.clone()],
                    },
                },
            })
            .unwrap();

            ("{string}");

            event_subscription(&mut self.subscriptions).insert(command_data, ch);

            trace!("sent {string}");

            // starting from here this could be done asynchronously
            // TODO FIXME I don't think we need the flushing requirement here specifically. maybe flush if no channel is ready or something like that
            self.stream
                .send(Message::Text(string))
                .await
                .map_err(crate::result::ErrorInner::WebSocket)?;
        };
        Ok(())
    }

    async fn handle_command_internal<C: Serialize + Debug + Send, R: Send>(
        &mut self,
        command_data: C,
        sender: oneshot::Sender<R>,
        respond_command_constructor: impl FnOnce(oneshot::Sender<R>) -> RespondCommand + Send,
    ) -> crate::result::Result<()> {
        self.id += 1;

        self.pending_commands
            .insert(self.id, respond_command_constructor(sender));

        let string = serde_json::to_string(&WebDriverBiDiRemoteEndCommand {
            id: self.id,
            command_data,
        })
        .unwrap();

        trace!("sent {string}");

        self.stream
            .send(Message::Text(string))
            .await
            .map_err(crate::result::ErrorInner::WebSocket)?;

        Ok(())
    }

    fn handle_message(&mut self, message: &str) -> crate::result::Result<()> {
        let jd = &mut serde_json::Deserializer::from_str(message);
        let parsed_message: WebDriverBiDiLocalEndMessage<Value> =
            serde_path_to_error::deserialize(jd)
                .map_err(crate::result::ErrorInner::ParseReceivedWithPath)?;
        match parsed_message {
            WebDriverBiDiLocalEndMessage::CommandResponse(
                WebDriverBiDiLocalEndCommandResponse { id, result },
            ) => {
                let respond_command = self
                    .pending_commands
                    .remove(&id)
                    .ok_or(crate::result::ErrorInner::ResponseWithoutRequest(id))?;

                send_response(self, result, respond_command)
            }
            WebDriverBiDiLocalEndMessage::ErrorResponse(
                WebDriverBiDiLocalEndMessageErrorResponse {
                    id: _,
                    error,
                    message: _,
                    stacktrace: _,
                    extensible: _,
                },
            ) => {
                eprintln!("error response received {error}"); // TODO FIXME propage to command if it has an id.

                // TODO unsubscribe, send error etc

                Ok(())
            }
            WebDriverBiDiLocalEndMessage::Event(event) => handle_event(self, event),
        }
    }
}
