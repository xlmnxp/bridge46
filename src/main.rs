mod utils;
use tokio::io;
use tokio::net::{TcpStream, TcpListener};
use env_logger::Builder;
use log::LevelFilter;

use utils::{get_sni_from_packet, resolve_addr};

pub const DNS_SERVER: &str = "1.1.1.1:53";
pub const BIND_ADDRESS: &str = "::";

async fn handle_https_connections(client: TcpStream, port: u16) {
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

async fn handle_http_connections(client: TcpStream, port: u16) {
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

async fn https_listener(bind_address: &str, port: u16) -> std::io::Result<()> {
    let listener: TcpListener = TcpListener::bind(format!("{}:{}", bind_address, port)).await.unwrap();
    log::info!("Listening on {}", listener.local_addr().unwrap());
    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    loop {
        let (client, _) = listener.accept().await.unwrap();
        let handle: tokio::task::JoinHandle<()> = tokio::spawn(async move {
            handle_https_connections(client, port).await;
        });
        handles.push(handle);
    }
}

async fn http_listener(bind_address: &str, port: u16) -> std::io::Result<()> {
    let listener: TcpListener = TcpListener::bind(format!("{}:{}", bind_address, port)).await.unwrap();
    log::info!("Listening on {}", listener.local_addr().unwrap());
    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    loop {
        let (client, _) = listener.accept().await.unwrap();
        let handle: tokio::task::JoinHandle<()> = tokio::spawn(async move {
            handle_http_connections(client, port).await;
        });
        handles.push(handle);
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    Builder::new()
    .filter(None, LevelFilter::Info)
    .init();

    let https_listener = tokio::spawn(https_listener(BIND_ADDRESS, 443));
    let http_listener = tokio::spawn(http_listener(BIND_ADDRESS, 80));

    let _ = https_listener.await.expect("https_listener failed");
    let _ = http_listener.await.expect("https_listener failed");
    Ok(())
}
