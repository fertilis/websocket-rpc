use clap::Parser;
use std::time::Instant;
use ws_rpc::{init_logger, Agent, AgentId, Message, MessageHeader, RequestId};

#[derive(Debug, Parser)]
#[clap(name = "client_example")]
#[clap(about = "Client example for the websocket rpc")]
struct Args {
    #[clap(short, long, default_value = "ws://127.0.0.1:8080")]
    router_url: String,

    #[clap(short, long, default_value = "2")]
    src_agent_id: u32,

    #[clap(short, long, default_value = "1")]
    dst_agent_id: u32,

    #[clap(short, long, default_value = "hello world")]
    message: String,
}

fn main() {
    init_logger(
        vec!["ws_rpc".to_string(), "client_example".to_string()],
        "Debug",
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let local_set = tokio::task::LocalSet::new();
    let fut = local_set.run_until(main_a());
    runtime.block_on(fut);
}

async fn main_a() {
    let args = Args::parse();
    let src_agent_id = AgentId(args.src_agent_id);
    let agent = Agent::new(src_agent_id, &args.router_url);
    agent.run_as_task();
    log::info!("Client with agent_id {} is running", src_agent_id.0);
    let dst_agent_id = AgentId(args.dst_agent_id);
    let handshake_message = Message {
        header: MessageHeader {
            src_agent_id: src_agent_id,
            dst_agent_id: AgentId(0),
            request_id: RequestId(0),
        },
        body: vec![],
    };
    agent.push(handshake_message);
    let message = Message {
        header: MessageHeader {
            src_agent_id,
            dst_agent_id,
            request_id: RequestId(123),
        },
        body: args.message.as_bytes().to_vec(),
    };
    agent.push(message);
    let start = Instant::now();
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let response = agent.pop();
        match response {
            None => {
                if start.elapsed().as_secs() > 1 {
                    log::error!("no response");
                    break;
                }
            }
            Some(response) => {
                let text = String::from_utf8(response.body).unwrap();
                log::info!("response: {}", text);
                break;
            }
        }
    }
}
