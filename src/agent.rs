use anyhow::anyhow;
use futures_util::{poll, SinkExt, StreamExt};
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::Poll;
use websocket_lite::{ClientBuilder, Opcode};

use crate::agent_id::AgentId;
use crate::message::Message;

/// Communicates to the [Router]
pub struct Agent {
    id: AgentId,
    router_url: String,
    inbound_queue: VecDeque<Message>,
    outbound_queue: VecDeque<Message>,
    _pin: std::marker::PhantomPinned,
}

impl Agent {
    pub fn new(id: AgentId, router_url: &str) -> Pin<Box<Self>> {
        let object = Self {
            id,
            router_url: router_url.to_string(),
            inbound_queue: VecDeque::new(),
            outbound_queue: VecDeque::new(),
            _pin: std::marker::PhantomPinned,
        };
        Box::pin(object)
    }

    pub fn push(self: &Pin<Box<Self>>, message: Message) {
        self.static_mut().outbound_queue.push_back(message);
    }

    pub fn pop(self: &Pin<Box<Self>>) -> Option<Message> {
        self.static_mut().inbound_queue.pop_front()
    }

    pub fn peek<'a>(self: &'a Pin<Box<Self>>) -> Option<&'a Message> {
        self.static_mut().inbound_queue.front()
    }

    pub fn run_as_task(self: &Pin<Box<Self>>) {
        let worker: &mut Self = self.static_mut();
        tokio::task::spawn_local(async move {
            worker.run().await;
        });
    }

    fn static_mut(self: &Pin<Box<Self>>) -> &'static mut Self {
        unsafe { &mut *(std::ptr::addr_of!(**self) as *mut Self) }
    }

    async fn run(&mut self) {
        loop {
            if let Err(e) = self.run_until_error().await {
                log::error!("Agent::run_until_error(): {:?}", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
        }
    }

    async fn run_until_error(&mut self) -> anyhow::Result<()> {
        log::info!("Connecting to: {}", self.router_url);
        let mut client = ClientBuilder::new(&self.router_url)?
            .async_connect()
            .await
            .map_err(|e| anyhow!("connect failed: {}", e))?;
        log::info!("Connected");

        loop {
            match poll!(client.next()) {
                Poll::Pending => {}
                Poll::Ready(None) => {
                    log::debug!("msg read got None");
                    return Ok(());
                }
                Poll::Ready(Some(Err(e))) => {
                    return Err(anyhow!("read failed: {}", e));
                }
                Poll::Ready(Some(Ok(message))) => match message.opcode() {
                    Opcode::Text => {}
                    Opcode::Binary => {
                        if let Ok(message) = Message::try_from(message.into_data().as_ref()) {
                            self.inbound_queue.push_back(message);
                        }
                    }
                    Opcode::Ping => {
                        client
                            .send(websocket_lite::Message::pong(message.into_data()))
                            .await
                            .map_err(|e| anyhow!("send failed: {}", e))?;
                    }
                    Opcode::Close => return Ok(()),
                    Opcode::Pong => {}
                },
            }
            while let Some(message) = self.outbound_queue.pop_front() {
                log::debug!("Sending message: {:?}", message);
                client
                    .send(websocket_lite::Message::binary(message.as_ref().to_vec()))
                    .await
                    .map_err(|e| anyhow!("send failed: {}", e))?;
                log::debug!("Sent");
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }
}
