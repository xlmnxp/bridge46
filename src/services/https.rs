use tls_parser::{parse_tls_extensions, parse_tls_plaintext};
use tokio::io;
use tokio::net::{TcpListener, TcpStream};

use crate::utils::resolve_addr;

pub fn get_sni_from_packet(packet: &[u8]) -> Option<String> {
    let parse_tls_plaintext = parse_tls_plaintext(&packet);
    if parse_tls_plaintext.is_err() {
        log::error!("Error parsing TLS packet: {:?}", parse_tls_plaintext.err());
        return None;
    }

    let tls_message = &parse_tls_plaintext.ok()?.1.msg[0];
    if let tls_parser::TlsMessage::Handshake(handshake) = tls_message {
        if let tls_parser::TlsMessageHandshake::ClientHello(client_hello) = handshake {
            let extensions: &[u8] = client_hello.ext?;
            let parsed_extensions = parse_tls_extensions(extensions).ok()?;
            for extension in parsed_extensions.1 {
                if let tls_parser::TlsExtension::SNI(sni) = extension {
                    return match String::from_utf8(sni[0].1.to_vec()) {
                        Ok(sni) => Some(sni),
                        Err(err) => {
                            log::error!("Error parsing SNI: {:?}", err);
                            None
                        }
                    };
                }
            }
        }
    }
    None
}

pub async fn handle_connection(client: TcpStream, port: u16) -> Option<()> {
    let src_addr = client.peer_addr().ok()?;

    let mut buf = [0; 2048];
    client.peek(&mut buf).await.expect("peek failed");
    if let Some(sni_string) = get_sni_from_packet(&buf) {
        if let Ok(ip) = resolve_addr(&sni_string).await {
            log::info!(
                "HTTPS {} Choose AAAA record for {}: {}",
                src_addr,
                sni_string,
                ip
            );

            let server: Result<TcpStream, io::Error> =
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
            tokio::spawn(async move { io::copy(&mut eread, &mut owrite).await });
            tokio::spawn(async move { io::copy(&mut oread, &mut ewrite).await });
            return Some(());
        }
    } else {
        log::error!("HTTPS {} No SNI", src_addr);
    }
    None
}

pub async fn listener(bind_address: &str, port: u16) -> std::io::Result<()> {
    let listener: TcpListener = TcpListener::bind(format!("{}:{}", bind_address, port)).await?;
    log::info!("Listening on {}", listener.local_addr()?);

    loop {
        let (client, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connection(client, port).await;
        });
    }
}
