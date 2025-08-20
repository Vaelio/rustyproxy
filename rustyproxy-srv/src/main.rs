use service::init_tracing;

mod handlers;

use crate::handlers::rpc_handler::handle_rpc;



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing("Tarpc Example Server")?;
    tokio::spawn(async {
        let _ = handle_rpc().await;
    });

    loop {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}