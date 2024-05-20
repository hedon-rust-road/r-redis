use tokio::net::TcpStream;

use crate::Backend;

pub async fn handle_stream(_stream: TcpStream, _backend: Backend) -> anyhow::Result<()> {
    Ok(())
}
