use anyhow::anyhow;
use bytes::BytesMut;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::{
    cmd::{Command, CommandExecutor},
    err::RespError,
    Backend, RespDecodeV2, RespEncode, RespFrame,
};

struct RespFrameCodec;

struct RedisRequest {
    frame: RespFrame,
    backend: Backend,
}

struct RedisResponse {
    frame: RespFrame,
}

impl Encoder<RespFrame> for RespFrameCodec {
    type Error = anyhow::Error;
    fn encode(&mut self, item: RespFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bs = item.encode();
        dst.extend_from_slice(bs.as_slice());
        Ok(())
    }
}

impl Decoder for RespFrameCodec {
    type Error = anyhow::Error;
    type Item = RespFrame;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<RespFrame>, Self::Error> {
        let res = RespFrame::decode(src);
        match res {
            Err(RespError::NotCompleted) => Ok(None),
            Ok(frame) => Ok(Some(frame)),
            Err(e) => Ok(Some(RespFrame::Error(e.to_string().into()))),
        }
    }
}

pub async fn handle_stream(stream: TcpStream, backend: Backend) -> anyhow::Result<()> {
    let mut framed = Framed::new(stream, RespFrameCodec);

    loop {
        match framed.next().await {
            None => return Err(anyhow!("connection closed")),
            Some(Err(e)) => return Err(anyhow!(e.to_string())),
            Some(Ok(frame)) => {
                let req = RedisRequest {
                    frame,
                    backend: backend.clone(),
                };
                let resp = handle_request(req).await?;
                framed.send(resp.frame).await?;
            }
        }
    }
}

async fn handle_request(req: RedisRequest) -> anyhow::Result<RedisResponse> {
    let (frame, backend) = (req.frame, req.backend);
    match TryInto::<Command>::try_into(frame) {
        Ok(cmd) => {
            let res = cmd.execute(&backend);
            Ok(RedisResponse { frame: res })
        }
        Err(e) => Ok(RedisResponse {
            frame: RespFrame::Error(e.to_string().into()),
        }),
    }
}
