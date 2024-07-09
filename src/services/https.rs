use tokio::io;
use tokio::net::{TcpStream, TcpListener};
use tls_parser::{parse_tls_extensions, parse_tls_plaintext};

use crate::utils::resolve_addr;

pub fn get_sni_from_packet(packet: &[u8]) -> Option<String> {
    let res: Result<
        (&[u8], tls_parser::TlsPlaintext),
        tls_parser::Err<tls_parser::nom::error::Error<&[u8]>>,
    > = parse_tls_plaintext(&packet);
    if res.is_err() {
        return None;
    }
    let tls_message: &tls_parser::TlsMessage = &res.unwrap().1.msg[0];
    if let tls_parser::TlsMessage::Handshake(handshake) = tls_message {
        if let tls_parser::TlsMessageHandshake::ClientHello(client_hello) = handshake {
            // get the extensions
            let extensions: &[u8] = client_hello.ext.unwrap();
            // parse the extensions
            let res: Result<
                (&[u8], Vec<tls_parser::TlsExtension>),
                tls_parser::Err<tls_parser::nom::error::Error<&[u8]>>,
            > = parse_tls_extensions(extensions);
            // iterate over the extensions and find the SNI
            for extension in res.unwrap().1 {
                if let tls_parser::TlsExtension::SNI(sni) = extension {
                    // get the hostname
                    let hostname: &[u8] = sni[0].1;
                    let s: String = match String::from_utf8(hostname.to_vec()) {
                        Ok(v) => v,
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };
                    return Some(s);
                }
            }
        }
    }
    None
}


pub async fn handle_connection(client: TcpStream, port: u16) {
    let src_addr = client.peer_addr().unwrap();

    let mut buf = [0; 1024];
    client.peek(&mut buf).await.expect("peek failed");
    let sni: Option<String> = get_sni_from_packet(&buf);
    if let Some(sni_string) = sni {
        if let Ok(ip) = resolve_addr(&sni_string).await {
            log::info!("HTTPS {} Choose AAAA record for {}: {}", src_addr, sni_string, ip);

            let server: Result<TcpStream, io::Error> = TcpStream::connect(format!("[{}]:{}", ip, port)).await;
            if server.is_err() {
                log::error!(
                    "HTTPS {} Failed to connect to upstream: {}",
                    src_addr, format!("{}:{}", ip, port)
                );
                return;
            }
            let server: TcpStream = server.unwrap();
            let (mut eread, mut ewrite) = client.into_split();
            let (mut oread, mut owrite) = server.into_split();
            log::info!("HTTPS {} Connected to upstream: {}", src_addr, format!("[{}]:{}", ip, port));
            tokio::spawn(async move { io::copy(&mut eread, &mut owrite).await });
            tokio::spawn(async move { io::copy(&mut oread, &mut ewrite).await });
        }
    } else {
        log::error!("HTTPS {} No SNI", src_addr);
    }
}

pub async fn listener(bind_address: &str, port: u16) -> std::io::Result<()> {
    let listener: TcpListener = TcpListener::bind(format!("{}:{}", bind_address, port)).await.unwrap();
    log::info!("Listening on {}", listener.local_addr().unwrap());
    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    loop {
        let (client, _) = listener.accept().await.unwrap();
        let handle: tokio::task::JoinHandle<()> = tokio::spawn(async move {
            handle_connection(client, port).await;
        });
        handles.push(handle);
    }
}
