use clap::Parser;
use ws_rpc::{init_logger, Agent, AgentId, Message, MessageHeader, RequestId};

#[derive(Debug, Parser)]
#[clap(name = "server_example")]
#[clap(about = "Server example for the websocket rpc")]
struct Args {
    #[clap(short, long, default_value = "ws://127.0.0.1:8080")]
    router_url: String,

    #[clap(short, long, default_value = "1")]
    agent_id: u32,
}

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let local_set = tokio::task::LocalSet::new();
    let fut = local_set.run_until(main_a());
    runtime.block_on(fut);
}

async fn main_a() {
    init_logger(
        vec!["ws_rpc".to_string(), "server_example".to_string()],
        "Debug",
    );
    let args = Args::parse();
    let agent_id = AgentId(args.agent_id);
    let agent = Agent::new(agent_id, &args.router_url);
    agent.run_as_task();
    log::info!("Server with agent_id {} is running", agent_id.0);
    let handshake_message = Message {
        header: MessageHeader {
            src_agent_id: agent_id,
            dst_agent_id: AgentId(0),
            request_id: RequestId(0),
        },
        body: vec![],
    };
    agent.push(handshake_message);
    loop {
        if let Some(request) = agent.pop() {
            let response: Message = handle_request(agent_id, request);
            agent.push(response);
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

fn handle_request(agent_id: AgentId, request: Message) -> Message {
    let body = format!(
        "I got request: {}, from: {}, with body: {}",
        request.header.request_id.0,
        request.header.src_agent_id.0,
        String::from_utf8_lossy(&request.body)
    );
    Message {
        header: MessageHeader {
            src_agent_id: agent_id,
            dst_agent_id: request.header.src_agent_id,
            request_id: request.header.request_id,
        },
        body: body.into_bytes(),
    }
}
