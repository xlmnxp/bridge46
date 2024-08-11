use tokio::io;
use tokio::net::{TcpListener, TcpStream};

use crate::utils::{get_bind_address, resolve_addr};

async fn handle_connection(client: TcpStream, port: u16) -> Option<()> {
    let src_addr = client.peer_addr().ok()?;

    // read request header and get the host
    let mut buf: [u8; 256] = [0; 256];
    client.peek(&mut buf).await.expect("peek failed");
    let mut request = String::from_utf8_lossy(&buf);
    let mut host: Option<String> = request
        .lines()
        .find(|line| line.to_lowercase().starts_with("host: "))
        .map(|line| String::from(line.to_lowercase().trim_start_matches("host: ").trim()));

    let mut fragment_buffer: [u8; 256] = [0; 256];
    let mut fragments: Vec<u8> = buf.to_vec();

    loop {
        if let Some(host_string) = host.clone() {
            let resolved_address: Result<std::net::IpAddr, io::Error> =
                resolve_addr(&host_string).await;
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
                if fragments.len() > 4096 || (fragments.len() > 0 && fragment_buffer.len() == 0) {
                    log::error!(
                        "HTTP {} Failed to resolve AAAA record for {}: {}",
                        src_addr,
                        host_string,
                        resolved_address.err()?
                    );
                    break;
                }

                fragment_buffer = [0; 256];

                client
                    .peek(&mut fragment_buffer)
                    .await
                    .expect("peek failed");
                fragments = [fragments, fragment_buffer.to_vec()].concat();
                request = String::from_utf8_lossy(fragments.as_slice());

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
