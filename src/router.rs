use bytes::BytesMut;
use futures_util::StreamExt;
use futures_util::{poll, SinkExt};
use std::pin::Pin;
use std::task::Poll;
use tokio::net::TcpStream;
use tokio::time::Instant;
use tokio_websockets::{Message, ServerBuilder, WebsocketStream};

/// Websocket server:
/// 1. Accepts connections from [Agent]s
/// 2. Reads messages from sending [Agent]s
/// 3. Writes messages to receiving [Agent]s
pub struct Router {
    port: u16,
    _pin: std::marker::PhantomPinned,
}

impl Router {
    pub fn new(port: u16) -> Pin<Box<Self>> {
        let object = Router {
            port,
            _pin: std::marker::PhantomPinned,
        };
        Box::pin(object)
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
        println!("Listening on port {}", self.port);
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = tokio::net::TcpListener::bind(addr).await?;
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
        log::info!("Client connected: {}", client_address);
        let mut last_ping: Instant = Instant::now();
        loop {
            match poll!(ws_stream.next()) {
                Poll::Ready(Some(Ok(_message))) => {
                    todo!()
                    // if message.is_text() {
                    //     let message: &str = message.as_text().unwrap();
                    //     if let Err(_) = handle_inbound_message(state, message, &client_address) {
                    //         log::error!("Invalid inbound message: {}", message);
                    //     }
                    // }
                }
                Poll::Ready(None) => {
                    log::info!("Client disconnected");
                    break;
                }
                Poll::Ready(Some(Err(e))) => {
                    log::error!("Error in reading message from client: {:?}", e);
                    break;
                }
                Poll::Pending => (),
            }

            let since_last_ping: tokio::time::Duration = last_ping.elapsed();
            if since_last_ping.as_secs() > 5 {
                ws_stream.send(Message::ping(BytesMut::new())).await?;
                last_ping = Instant::now();
                continue;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        Ok(())
    }
}
