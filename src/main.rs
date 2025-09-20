mod services;
mod utils;
use std::vec;

use env_logger::Builder;
use log::LevelFilter;
use services::http;
use services::https;
use services::minecraft;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    Builder::new().filter(None, LevelFilter::Info).init();

    let http_ports: vec::Vec<u16> = vec![80, 8080, 10000];
    let https_ports: vec::Vec<u16> = vec![443, 8443, 10443];

    for http_port in http_ports {
        let listner = tokio::spawn(http::listener(http_port));
        let _ = listner.await.expect("http listener failed");
    }

    for https_port in https_ports {
        let listner = tokio::spawn(https::listener(https_port));
        let _ = listner.await.expect("https listener failed");
    }

    let minecraft_listener: tokio::task::JoinHandle<Result<(), std::io::Error>> = tokio::spawn(minecraft::listener(25565));

    let _ = minecraft_listener.await.expect("minecraft_listener failed");

    Ok(())
}
