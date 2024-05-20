use rredis::{network, Backend};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:6379";
    info!("R-Redis is running on {}", addr);
    let listener = TcpListener::bind(addr).await?;

    let backend = Backend::new();
    loop {
        let (stream, socket_addr) = listener.accept().await?;
        info!("Accepted connection from {}", socket_addr);
        let cloned_backend = backend.clone();
        tokio::spawn(async move {
            match network::handle_stream(stream, cloned_backend).await {
                Ok(_) => {
                    info!("Connection from {} exited", socket_addr);
                }
                Err(e) => {
                    info!("Error handling connection from {}: {}", socket_addr, e);
                }
            }
        });
    }
}
