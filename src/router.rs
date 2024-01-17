use bytes::{Bytes, BytesMut};
use futures_util::StreamExt;
use futures_util::{poll, SinkExt};
use std::pin::Pin;
use std::task::Poll;
use tokio::net::TcpStream;
use tokio::time::Instant;
use tokio_websockets::{Message as TokioMessage, ServerBuilder, WebsocketStream};

use crate::agent_id::AgentId;
use crate::message::Message;
use crate::message_queues::MessageQueues;

/// Websocket server:
/// 1. Accepts connections from [Agent]s
/// 2. Reads messages from sending [Agent]s
/// 3. Writes messages to receiving [Agent]s
pub struct Router {
    port: u16,
    message_queues: MessageQueues,
    _pin: std::marker::PhantomPinned,
}

impl Router {
    pub fn new(port: u16) -> Pin<Box<Self>> {
        let object = Router {
            port,
            message_queues: MessageQueues::new(),
            _pin: std::marker::PhantomPinned,
        };
        Box::pin(object)
    }

    pub async fn run_forever(self: &Pin<Box<Self>>) {
        let worker: &'static mut Self = self.static_mut();
        worker.run().await;
    }

    pub fn run_as_task(self: &Pin<Box<Self>>) {
        let worker: &'static mut Self = self.static_mut();
        tokio::task::spawn_local(async move {
            worker.run().await;
        });
    }

    fn static_mut(self: &Pin<Box<Self>>) -> &'static mut Self {
        unsafe { &mut *(std::ptr::addr_of!(**self) as *mut Self) }
    }

    fn static_mut_2(&mut self) -> &'static mut Self {
        unsafe { &mut *(std::ptr::addr_of!(*self) as *mut Self) }
    }

    async fn run(&mut self) {
        loop {
            if let Err(e) = self.run_until_error().await {
                log::error!("Router::run_until_error(): {:?}", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
        }
    }

    async fn run_until_error(&mut self) -> anyhow::Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        log::info!("Listening on: {}", listener.local_addr()?);
        while let Ok((stream, client_address)) = listener.accept().await {
            let ws_stream: WebsocketStream<TcpStream> = ServerBuilder::new().accept(stream).await?;
            let client_address: String = client_address.to_string();
            let worker: &'static mut Self = self.static_mut_2();
            tokio::task::spawn_local(async move {
                if let Err(e) = worker.serve_connection(ws_stream, client_address).await {
                    log::error!("serve_connection(): {:?}", e);
                }
            });
        }
        Ok(())
    }

    async fn serve_connection(
        &mut self,
        mut ws_stream: WebsocketStream<TcpStream>,
        client_address: String,
    ) -> anyhow::Result<()> {
        log::debug!("Client connected: {}", client_address);
        let mut agent_id: Option<AgentId> = None;
        let mut last_ping: Instant = Instant::now();
        loop {
            match poll!(ws_stream.next()) {
                Poll::Ready(Some(Ok(message))) => {
                    if message.is_binary() {
                        let payload: Bytes = message.into_payload();
                        let payload: &[u8] = payload.as_ref();
                        if let Ok(message) = Message::try_from(payload) {
                            if message.is_handshake() {
                                agent_id = Some(message.header.src_agent_id);
                                log::debug!("Got handshake from: {}", agent_id.unwrap().0);
                                log::debug!("Handshake: {:?}", &message);
                            } else {
                                log::debug!("Got message: {:?}", &message);
                                self.message_queues.push(message);
                            }
                        }
                    }
                }
                Poll::Ready(None) => {
                    log::debug!("Client disconnected");
                    break;
                }
                Poll::Ready(Some(Err(e))) => {
                    log::error!("Error in reading message from client: {:?}", e);
                    break;
                }
                Poll::Pending => (),
            }
            match agent_id {
                None => (),
                Some(agent_id) => {
                    while let Some(message) = self.message_queues.pop(agent_id) {
                        let message_bytes: Vec<u8> = message.into();
                        ws_stream
                            .send(TokioMessage::binary(BytesMut::from(
                                message_bytes.as_slice(),
                            )))
                            .await?;
                        log::debug!("Sent message to: {}", agent_id.0);
                    }
                }
            }
            let since_last_ping: tokio::time::Duration = last_ping.elapsed();
            if since_last_ping.as_secs() > 5 {
                ws_stream.send(TokioMessage::ping(BytesMut::new())).await?;
                last_ping = Instant::now();
                continue;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        Ok(())
    }
}
