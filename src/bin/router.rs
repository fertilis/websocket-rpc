use clap::Parser;
use ws_rpc::{init_logger, Router};

#[derive(Debug, Parser)]
#[clap(name = "router")]
#[clap(about = "Router for the websocket rpc")]
struct Args {
    #[clap(short, long, default_value = "8080")]
    port: u16,
}

fn main() {
    init_logger(vec!["ws_rpc".to_string()], "Debug");
    let args = Args::parse();
    let router = Router::new(args.port);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let local_set = tokio::task::LocalSet::new();
    let fut = local_set.run_until(router.run_forever());
    runtime.block_on(fut);
}
