use std::collections::HashMap;
use std::fmt::Debug;

use futures::{SinkExt as _, StreamExt as _};
use serde::Serialize;
use serde_json::Value;
use tempfile::TempDir;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{error, trace};

use crate::browsing_context::BrowsingContext;
use crate::generated::{
    handle_command, handle_event, send_response, EventSubscription, GlobalEventSubscription,
    RespondCommand, SendCommand,
};
use crate::protocol::{self, Command, CommandResponse, Extensible};
use crate::session;

pub struct WebDriverHandler {
    id: u64,
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    receive_command: mpsc::UnboundedReceiver<SendCommand>,
    pending_commands: HashMap<u64, RespondCommand>,
    pub(crate) subscriptions: EventSubscription,
    pub(crate) global_subscriptions: GlobalEventSubscription,
    // ensure the profile folder is not deleted
    _tmp_dir: TempDir,
}

impl WebDriverHandler {
    pub async fn handle(
        tmp_dir: TempDir,
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
            _tmp_dir: tmp_dir,
        };
        this.handle_internal().await;
    }

    async fn handle_internal(&mut self) {
        loop {
            trace!("select");
            tokio::select! {
                // TODO FIXME make this truly parallel. e.g. if receiving a message while sending hangs
                message = self.stream.next() => {
                    match message {
                        Some(Ok(Message::Text(message))) => {
                            trace!("received {message}");
                            if let Err(error) = self.handle_message(&message) {
                                error!("error when parsing incoming message {message} {error}");
                            }
                        }
                        Some(Ok(message)) => {
                            error!("Unknown message: {message}");
                        }
                        Some(Err(error)) => error!("Error in receive {error}"),
                        None => {
                            error!("connection closed");
                            break;
                        }
                    }
                }
                // TODO FIXME use the receive many functions
                Some(receive_command) = self.receive_command.recv() => {
                    if let Err(error) = handle_command(self, receive_command).await {
                        error!("error when handling incoming command {error}");
                    }
                }
            }
        }
        error!("handle closed");
    }

    pub(crate) async fn handle_global_subscription_internal<R: Clone + Send>(
        &mut self,
        event: String,
        sender: oneshot::Sender<broadcast::Receiver<R>>,
        global_event_subscription: impl Fn(
                &mut GlobalEventSubscription,
            ) -> &mut Option<(broadcast::Sender<R>, broadcast::Receiver<R>)>
            + Send,
        respond_command_constructor: impl FnOnce(
                broadcast::Receiver<R>,
                oneshot::Sender<broadcast::Receiver<R>>,
            ) -> RespondCommand
            + Send,
    ) -> crate::error::Result<()> {
        match global_event_subscription(&mut self.global_subscriptions) {
            Some(subscription) => {
                sender
                    .send(subscription.0.subscribe())
                    .map_err(|_| crate::error::ErrorInner::CommandCallerExited)?;
            }
            None => {
                self.id += 1;

                let ch = broadcast::channel(10);

                self.pending_commands.insert(
                    self.id,
                    respond_command_constructor(ch.0.subscribe(), sender),
                );

                let string = serde_json::to_string(&protocol::Command {
                    id: self.id,
                    data: session::subscribe::Command {
                        params: session::SubscriptionRequest {
                            events: vec![event],
                            contexts: None,
                        },
                    },
                    extensible: Extensible::default(),
                })
                .unwrap();

                *global_event_subscription(&mut self.global_subscriptions) = Some(ch);

                trace!("sent {string}");

                // starting from here this could be done asynchronously
                // TODO FIXME I don't think we need the flushing requirement here specifically. maybe flush if no channel is ready or something like that
                self.stream
                    .send(Message::Text(string))
                    .await
                    .map_err(crate::error::ErrorInner::WebSocket)?;
                self.stream
                    .flush()
                    .await
                    .map_err(crate::error::ErrorInner::WebSocket)?;
            }
        };
        Ok(())
    }

    pub(crate) async fn handle_subscription_internal<R: Clone + Send>(
        &mut self,
        event: String,
        command_data: BrowsingContext,
        sender: oneshot::Sender<broadcast::Receiver<R>>,
        event_subscription: impl Fn(
                &mut EventSubscription,
            )
                -> &mut HashMap<BrowsingContext, (broadcast::Sender<R>, broadcast::Receiver<R>)>
            + Send,
        respond_command_constructor: impl FnOnce(
                broadcast::Receiver<R>,
                oneshot::Sender<broadcast::Receiver<R>>,
            ) -> RespondCommand
            + Send,
    ) -> crate::error::Result<()> {
        if let Some(subscription) = event_subscription(&mut self.subscriptions).get(&command_data) {
            sender
                .send(subscription.0.subscribe())
                .map_err(|_| crate::error::ErrorInner::CommandCallerExited)?; // TODO FIXME this would return before the request command is actually done
        } else {
            self.id += 1;

            let ch = broadcast::channel(10);

            self.pending_commands.insert(
                self.id,
                respond_command_constructor(ch.0.subscribe(), sender),
            );

            let string = serde_json::to_string(&protocol::Command {
                id: self.id,
                data: session::subscribe::Command {
                    params: session::SubscriptionRequest {
                        events: vec![event],
                        contexts: Some(vec![command_data.clone()]),
                    },
                },
                extensible: Extensible::default(),
            })
            .unwrap();

            event_subscription(&mut self.subscriptions).insert(command_data, ch);

            trace!("sent {string}");

            // starting from here this could be done asynchronously
            // TODO FIXME I don't think we need the flushing requirement here specifically. maybe flush if no channel is ready or something like that
            self.stream
                .send(Message::Text(string))
                .await
                .map_err(crate::error::ErrorInner::WebSocket)?;
            self.stream
                .flush()
                .await
                .map_err(crate::error::ErrorInner::WebSocket)?;
        };
        Ok(())
    }

    pub(crate) async fn handle_command_internal<C: Serialize + Debug + Send, R: Send>(
        &mut self,
        command_data: C,
        sender: oneshot::Sender<R>,
        respond_command_constructor: impl FnOnce(oneshot::Sender<R>) -> RespondCommand + Send,
    ) -> crate::error::Result<()> {
        self.id += 1;

        self.pending_commands
            .insert(self.id, respond_command_constructor(sender));

        let string = serde_json::to_string(&Command {
            id: self.id,
            data: command_data,
            extensible: Extensible::default(),
        })
        .unwrap();

        trace!("sent {string}");

        self.stream
            .send(Message::Text(string))
            .await
            .map_err(crate::error::ErrorInner::WebSocket)?;
        self.stream
            .flush()
            .await
            .map_err(crate::error::ErrorInner::WebSocket)?;

        Ok(())
    }

    fn handle_message(&mut self, message: &str) -> crate::error::Result<()> {
        let jd = &mut serde_json::Deserializer::from_str(message);
        let parsed_message: protocol::Message<Value> = serde_path_to_error::deserialize(jd)
            .map_err(crate::error::ErrorInner::ParseReceivedWithPath)?;
        match parsed_message {
            protocol::Message::CommandResponse(CommandResponse { id, result }) => {
                let respond_command = self
                    .pending_commands
                    .remove(&id)
                    .ok_or(crate::error::ErrorInner::ResponseWithoutRequest(id))?;

                send_response(self, result, respond_command)
            }
            protocol::Message::ErrorResponse(error_response) => {
                error!("error response received {error_response:?}"); // TODO FIXME propage to command if it has an id.

                // TODO unsubscribe, send error etc

                Ok(())
            }
            protocol::Message::Event(event) => handle_event(self, event),
        }
    }
}
