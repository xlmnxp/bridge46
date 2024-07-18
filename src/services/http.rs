use tokio::io;
use tokio::net::{TcpListener, TcpStream};

use crate::utils::{get_bind_address, resolve_addr};

async fn handle_connection(client: TcpStream, port: u16) -> Option<()> {
    let src_addr = client.peer_addr().ok()?;

    // read request header and get the host
    let mut buf = [0; 1024];
    client.peek(&mut buf).await.expect("peek failed");
    let request = String::from_utf8_lossy(&buf);
    let host: Option<&str> = request
        .lines()
        .find(|line| line.starts_with("Host: "))
        .map(|line| line.trim_start_matches("Host: ").trim());
    if let Some(host_string) = host {
        let resolved_address: Result<std::net::IpAddr, io::Error> = resolve_addr(&host_string).await;
        if let Ok(ip) = resolved_address {
            log::info!(
                "HTTP {} Choose AAAA record for {}: {}",
                src_addr,
                host_string,
                ip
            );

            let server: Result<TcpStream, io::Error> =
                TcpStream::connect(format!("[{}]:{}", ip, port)).await;
            if server.is_err() {
                log::error!(
                    "HTTP {} Failed to connect to upstream: {}",
                    src_addr,
                    format!("{}:{}", ip, port)
                );
                return None;
            }

            let server: TcpStream = server.ok()?;
            let (mut eread, mut ewrite) = client.into_split();
            let (mut oread, mut owrite) = server.into_split();
            log::info!(
                "HTTP {} Connected to upstream: {}",
                src_addr,
                format!("[{}]:{}", ip, port)
            );
            tokio::spawn(async move { io::copy(&mut eread, &mut owrite).await });
            tokio::spawn(async move { io::copy(&mut oread, &mut ewrite).await });
            return Some(());
        } else {
            log::error!(
                "HTTPS {} Failed to resolve AAAA record for {}: {}",
                src_addr,
                host_string,
                resolved_address.err()?
            );
        }
    }
    None
}

pub async fn listener(port: u16) -> std::io::Result<()> {
    let listener: TcpListener = TcpListener::bind(format!("{}:{}", get_bind_address(), port)).await?;
    log::info!("Listening on {}", listener.local_addr()?);

    loop {
        let (client, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connection(client, port).await;
        });
    }
}
