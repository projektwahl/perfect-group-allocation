use core::hash::Hash;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use futures::{SinkExt as _, StreamExt as _};
use serde::Serialize;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

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
        /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
        #[derive(Debug)]
        pub enum SendCommand {
            $(#[doc = $doc] $variant($($command)::*::Command, oneshot::Sender<$($command)::*::Result>),)*
            /// the list should not be empty because then you subscribe to nothing
            $(#[doc = $doc_subscription] $variant_subscription(Option<Vec<BrowsingContext>>, oneshot::Sender<broadcast::Receiver<$($command_subscription)::*>>),)*
        }

        /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
        #[derive(Debug)]
        pub enum RespondCommand {
            $(#[doc = $doc] $variant(oneshot::Sender<$($command)::*::Result>),)*
            $(#[doc = $doc_subscription] $variant_subscription(oneshot::Sender<broadcast::Receiver<$($command_subscription)::*>>),)*
        }

        /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
        #[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(tag = "method")]
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
        pub enum EventData {
            $(
                #[doc = $doc_subscription]
                #[serde(rename = $tag_subscription)]
                $variant_subscription($($command_subscription)::*)
            ,)*
        }

         /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
         #[derive(Debug)]
         pub enum GlobalEventSubscription {
             $(
                #[doc = $doc_subscription]
                $variant_subscription(Option<(broadcast::Sender<$($command_subscription)::*>, broadcast::Receiver<$($command_subscription)::*>)>)
             ,)*
         }

        /// <https://w3c.github.io/webdriver-bidi/#protocol-definition>
        #[derive(Debug, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(tag = "method")]
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
                        this.handle_global_subscription_internal(command, sender, RespondCommand::$variant_subscription).await?;
                    }
                ),*
            }
            Ok(())
        }

        fn handle_event(this: &mut WebDriverHandler, input: EventData) -> crate::result::Result<()> {
            match input {
                $(
                    EventData::$variant_subscription(event) => {
                        // TODO FIXME don't unwrap but unsubscribe in this case
                        // TODO FIXME extract method
                       // this.$subscription_store.as_ref().unwrap().0.send(event).unwrap();
                    }
                ),*
            }
            Ok(())
        }

        fn send_response(this: &mut WebDriverHandler, result: Value, respond_command: RespondCommand) -> crate::result::Result<()> {
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
                    RespondCommand::$variant_subscription(respond_command) => {
                        // result here is the result of the subscribe command which should be empty
                        // serde_path_to_error::deserialize(result).map_err(crate::result::Error::ParseReceivedWithPath)?
                        //respond_command
                        //    .send(this.$subscription_store.as_mut().unwrap().0.subscribe())
                        //    .map_err(|_| crate::result::ErrorInner::CommandCallerExited)?
                    }
                ),*
            }
            Ok(())
        }
    };
}

impl PartialEq for GlobalEventSubscription {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl Eq for GlobalEventSubscription {}

impl Hash for GlobalEventSubscription {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Borrow<String> for GlobalEventSubscription {
    fn borrow(&self) -> &String {
        todo!()
    }
}

magic! {
    pub enum {
        /// https://w3c.github.io/webdriver-bidi/#command-session-new
        SessionNew("session.new" session::new),
        /// https://w3c.github.io/webdriver-bidi/#command-session-end
        SessionEnd("session.end" session::end),
        /// https://w3c.github.io/webdriver-bidi/#command-session-subscribe
        SessionSubscribe("session.subscribe" session::subscribe), // TODO FIXME this should not be in sendcommand
        /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree>
        BrowsingContextGetTree("browsingContext.getTree" browsing_context::get_tree),
        /// <https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate>
        BrowsingContextNavigate("browsingContext.navigate" browsing_context::navigate)
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
    // we need an inverted index because every client should get a single broadcast channel with the browsing contexts
    // they are interested about and therefore events need to be sent to multiple channels
    magic: HashMap<(String, BrowsingContext), Vec<broadcast::Sender<String>>>,
    log_subscriptions: HashMap<
        (String, BrowsingContext),
        (
            broadcast::Sender<log::EntryAdded>,
            broadcast::Receiver<log::EntryAdded>,
        ),
    >,
    global_subscriptions: HashSet<GlobalEventSubscription>,
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
            magic: HashMap::default(),
            pending_commands: HashMap::default(),
            log_subscriptions: HashMap::default(),
            global_subscriptions: HashSet::default(),
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

    async fn handle_global_subscription_internal<C: Serialize + Debug + Send, R: Clone + Send>(
        &mut self,
        command_data: C,
        sender: oneshot::Sender<broadcast::Receiver<R>>,
        respond_command_constructor: impl FnOnce(
            oneshot::Sender<broadcast::Receiver<R>>,
        ) -> RespondCommand
        + Send,
    ) -> crate::result::Result<()> {
        match self.global_subscriptions.get(&"test".to_string()) {
            Some(subscription) => {
                sender.send(subscription.0.subscribe());
            }
            None => {
                self.id += 1;

                self.pending_commands
                    .insert(self.id, respond_command_constructor(sender));

                let string = serde_json::to_string(&WebDriverBiDiRemoteEndCommand {
                    id: self.id,
                    command_data: session::subscribe::Command {
                        params: session::SubscriptionRequest {
                            events: vec!["log.entryAdded".to_owned()],
                            contexts: vec![],
                        },
                    },
                })
                .unwrap();

                println!("{string}");

                global_subscriptions(self).insert(broadcast::channel(10));

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

        println!("{string}");

        self.stream
            .send(Message::Text(string))
            .await
            .map_err(crate::result::ErrorInner::WebSocket)?;

        Ok(())
    }

    fn handle_message(&mut self, message: &str) -> crate::result::Result<()> {
        println!("{message}");
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

                Ok(())
            }
            WebDriverBiDiLocalEndMessage::Event(event) => handle_event(self, event),
        }
    }
}
