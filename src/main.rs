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
        tokio::spawn(http::listener(http_port));
    }

    for https_port in https_ports {
        tokio::spawn(https::listener(https_port));
    }

    // Minecraft listener
    tokio::spawn(minecraft::listener(25565));

    // Wait forever
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
