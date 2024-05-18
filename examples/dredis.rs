use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (mut socket, socket_addr) = listener.accept().await?;
        println!("accept connection from: {}", socket_addr);

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        println!("read error: {}", e);
                        break;
                    }
                };
                info!("{:?}", String::from_utf8_lossy(&buf[..n]));
                if let Err(e) = socket.write_all(b"+OK\r\n").await {
                    error!("write error: {}", e);
                    break;
                }
            }
        });
    }
}

// set hello world                  *3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n
// get hello                        *2\r\n$3\r\nget\r\n$5\r\nhello\r\n
// hset h1 hello world              *4\r\n$2\r\nhset\r\n$2\r\nh1\r\n$5\r\nhello\r\n$5\r\nworld\r\n
// hget h1 hello                    *3\r\n$2\r\nhget\r\n$2\r\nh1\r\n$5\r\nhello\r\n
// hmget h1 hello hello2 hello3     *5\r\n$5\r\nhmget\r\n$2\r\nh1\r\n$5\r\nhello\r\n$6\r\nhello2\r\n$6\r\nhello3\r\n
// hgetall h1                       *2\r\n$7\r\nhgetall\r\n$2\r\nh1\r\n
// echo "hello world"               *2\r\n$4\r\necho\r\n$11\r\nhello world\r\n
// sadd myset "one"                 *4\r\n$4\r\nsadd\r\n$5\r\nmyset\r\n$3\r\none\r\n
// sismember myset "one"            *3\r\n$9\r\nsismember\r\n$5\r\nmyset\r\n$3\r\none\r\n
