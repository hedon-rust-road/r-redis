use anyhow::anyhow;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    err::RespError, Backend, Command, CommandExecutor, RespDecode, RespEncode, RespFrame,
    SimpleError,
};

pub async fn handle_stream(mut stream: TcpStream, backend: Backend) -> anyhow::Result<()> {
    let mut buf = BytesMut::with_capacity(4096);

    loop {
        let n = stream.read_buf(&mut buf).await?;
        if n == 0 {
            return Err(anyhow!("connection closed"));
        }

        let res = RespFrame::decode(&mut buf);
        match res {
            Err(RespError::NotCompleted) => {
                continue;
            }
            Ok(frame) => {
                match handle_frame(frame, &mut stream, &backend).await {
                    Ok(_) => continue,
                    Err(e) => {
                        let e = SimpleError::new(e.to_string());
                        stream.write_all(e.encode().as_slice()).await?
                    }
                };
            }
            Err(e) => {
                let e = SimpleError::new(e.to_string());
                stream.write_all(e.encode().as_slice()).await?
            }
        }
    }
}

async fn handle_frame(
    frame: RespFrame,
    stream: &mut TcpStream,
    backend: &Backend,
) -> anyhow::Result<()> {
    let cmd: Command = frame.try_into()?;
    let res = cmd.execute(backend);
    let res = res.encode();
    stream.write_all(res.as_slice()).await?;
    Ok(())
}
