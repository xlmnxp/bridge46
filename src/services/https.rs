use tokio::net::{TcpListener, TcpStream};
use rustls::server::{Accepted, Acceptor};
use crate::utils::{get_bind_address, resolve_addr};

pub async fn get_sni_from_packet(packet: Vec<u8>) -> Option<String> {
    let mut acceptor: Acceptor = Acceptor::default();
    let cursor: &mut dyn std::io::Read = &mut &packet[..];

    match &acceptor.read_tls(cursor) {
        Ok(size) => {
            if *size == 0 {
                log::error!("No data read from TLS packet");
                return None;
            }
        }
        Err(err) => {
            log::error!("Error reading TLS packet: {:?}", err);
            return None;
        }
    };

    let accepted: Accepted = match acceptor.accept() {
        Ok(Some(acceptor)) => acceptor,
        Err(err) => {
            log::error!("Error processing new packets: {:?}", err);
            return None;
        }
        _ => {
            log::error!("Packet not enough to process SNI (will increase buffer size by 256 bytes)");
            return None;
        }
    };
    return match accepted.client_hello().server_name() {
        Some(sni) => Some(sni.to_string()),
        None => {
            log::error!("No SNI found in packet");
            None
        }
    }
}

pub async fn handle_connection(client: TcpStream, port: u16) -> Option<()> {
    let src_addr = client.peer_addr().ok()?;

    let mut buf: Vec<u8> = vec![0; 256];
    let mut last_buf_read_len = client.peek(&mut buf).await.expect("peek failed");

    loop {
        if let Some(sni_string) = get_sni_from_packet(buf.clone()).await {
            let resolved_address: Result<std::net::IpAddr, tokio::io::Error> =
                resolve_addr(&sni_string).await;
            if let Ok(ip) = resolved_address {
                log::info!(
                    "HTTPS {} Choose AAAA record for {}: {}",
                    src_addr,
                    sni_string,
                    ip
                );

                let server: Result<TcpStream, tokio::io::Error> =
                    TcpStream::connect(format!("[{}]:{}", ip, port)).await;
                if server.is_err() {
                    log::error!(
                        "HTTPS {} Failed to connect to upstream: {}",
                        src_addr,
                        format!("{}:{}", ip, port)
                    );
                    return None;
                }

                let server: TcpStream = server.ok()?;
                let (mut eread, mut ewrite) = client.into_split();
                let (mut oread, mut owrite) = server.into_split();
                log::info!(
                    "HTTPS {} Connected to upstream: {}",
                    src_addr,
                    format!("[{}]:{}", ip, port)
                );
                tokio::spawn(async move { tokio::io::copy(&mut eread, &mut owrite).await });
                tokio::spawn(async move { tokio::io::copy(&mut oread, &mut ewrite).await });
                return Some(());
            } else {
                log::error!(
                    "HTTPS {} Failed to resolve AAAA record for {}: {}",
                    src_addr,
                    sni_string,
                    resolved_address.err()?
                );
                break;
            }
        } else {
            if buf.len() > 4096 || last_buf_read_len == 0 {
                log::error!("HTTPS {} No SNI", src_addr);
                break;
            }

            let buf_new_len = buf.len() + 256;

            buf = vec![0; buf_new_len];
            buf.resize(buf_new_len, 0);

            last_buf_read_len = client
                .peek(&mut buf[..buf_new_len])
                .await
                .expect("peek failed");
            continue;
        }
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
