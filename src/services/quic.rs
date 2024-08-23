use std::io::Result;

use rustls::server::{Accepted, Acceptor};
use tokio::net::UdpSocket;
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

pub async fn listener(port: u16) -> Result<()> {
    let bind_address = format!("{}:{}", get_bind_address(), port);
    let listener = UdpSocket::bind(bind_address.clone()).await?;
    log::info!("QUIC Listening on: {}", bind_address);

    loop {
        let mut buf = vec![0; 2048];
        let (len, src) = listener.peek_from(&mut buf).await?;
        let packet = buf[..len].to_vec();

        if let Some(sni_string) = get_sni_from_packet(packet.clone()).await {
            let resolved_address: Result<std::net::IpAddr> =
                resolve_addr(&sni_string).await;
            if let Ok(ip) = resolved_address {
                log::info!(
                    "QUIC {} Choose AAAA record for {}: {}",
                    src,
                    sni_string,
                    ip
                );
                let server = UdpSocket::bind("::").await?;
                server.connect(format!("[{}]:{}", ip, port)).await?;
                server.send(&packet).await?;
            } else {
                log::error!(
                    "QUIC {} Failed to resolve AAAA record for {}: {}",
                    src,
                    sni_string,
                    resolved_address.err().unwrap()
                );
            }
        }
    }
}
