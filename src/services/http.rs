use tokio::io;
use tokio::net::{TcpStream, TcpListener};

use crate::utils::resolve_addr;

async fn handle_connection(client: TcpStream, port: u16) {
    let src_addr = client.peer_addr().unwrap();
    
    // read request header and get the host
    let mut buf = [0; 1024];
    client.peek(&mut buf).await.expect("peek failed");
    let request = String::from_utf8_lossy(&buf);
    let host: Option<&str> = request.lines().find(|line| line.starts_with("Host: ")).map(|line| line.trim_start_matches("Host: ").trim());
    if let Some(host_string) = host {
        if let Ok(ip) = resolve_addr(host_string).await {
            log::info!("HTTP {} Choose AAAA record for {}: {}", src_addr, host_string, ip);

            let server: Result<TcpStream, io::Error> = TcpStream::connect(format!("[{}]:{}", ip, port)).await;
            if server.is_err() {
                log::error!(
                    "HTTP {} Failed to connect to upstream: {}",
                    src_addr, format!("{}:{}", ip, port)
                );
                return;
            }
            let server: TcpStream = server.unwrap();
            let (mut eread, mut ewrite) = client.into_split();
            let (mut oread, mut owrite) = server.into_split();
            log::info!("HTTP {} Connected to upstream: {}", src_addr, format!("[{}]:{}", ip, port));
            tokio::spawn(async move { io::copy(&mut eread, &mut owrite).await });
            tokio::spawn(async move { io::copy(&mut oread, &mut ewrite).await });
        }
    }
}

pub async fn listener(bind_address: &str, port: u16) -> std::io::Result<()> {
    let listener: TcpListener = TcpListener::bind(format!("{}:{}", bind_address, port)).await.unwrap();
    log::info!("Listening on {}", listener.local_addr().unwrap());

    loop {
        let (client, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_connection(client, port).await;
        });
    }
}
