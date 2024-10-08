use tokio::net::{TcpListener, TcpStream};
use crate::utils::{forward, get_bind_address, resolve_addr};

async fn handle_connection(client: TcpStream, port: u16) -> Option<()> {
    let src_addr = client.peer_addr().ok()?;

    // read request header and get the host
    let mut buf: Vec<u8> = vec![0; 256];
    let mut last_buf_read_len = client.peek(&mut buf).await.expect("peek failed");

    let request_buf = buf.clone();
    let mut request = String::from_utf8_lossy(&request_buf);
    let mut host: Option<String> = request
        .lines()
        .find(|line| line.to_lowercase().starts_with("host: "))
        .map(|line| String::from(line.to_lowercase().trim_start_matches("host: ").trim()));

    loop {
        if let Some(host_string) = host.clone() {
            let resolved_address: Result<std::net::IpAddr, tokio::io::Error> =
                resolve_addr(&host_string).await;
            if let Ok(ip) = resolved_address {
                let distance = format!("[{}]:{}", ip, port);

                log::info!(
                    "HTTP {} Choose AAAA record for {}: {}",
                    src_addr,
                    host_string,
                    distance
                );

                return forward(client, distance).await;
            } else {
                if buf.len() > 4096 || last_buf_read_len == 0 {
                    log::error!(
                        "HTTP {} Failed to resolve AAAA record for {}: {}",
                        src_addr,
                        host_string,
                        resolved_address.err()?
                    );
                    break;
                }

                let buf_new_len = buf.len() + 256;
                buf = vec![0; buf_new_len];

                last_buf_read_len = client
                    .peek(&mut buf)
                    .await
                    .expect("peek failed");

                request = String::from_utf8_lossy(&buf);

                host = request
                    .lines()
                    .find(|line| line.to_lowercase().starts_with("host: "))
                    .map(|line| {
                        String::from(line.to_lowercase().trim_start_matches("host: ").trim())
                    });
                continue;
            }
        }
        break;
    }
    None
}

pub async fn listener(port: u16) -> std::io::Result<()> {
    let listener: TcpListener =
        TcpListener::bind(format!("{}:{}", get_bind_address(), port)).await?;
    log::info!("Listening on {}", listener.local_addr()?);

    loop {
        let (client, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connection(client, port).await;
        });
    }
}
