mod services;
mod utils;
use env_logger::Builder;
use log::LevelFilter;
use services::http;
use services::https;
use services::minecraft;
use services::smtp;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    Builder::new().filter(None, LevelFilter::Info).init();

    let http_listener = tokio::spawn(http::listener(80));
    let https_listener = tokio::spawn(https::listener(443));
    let minecraft_listener = tokio::spawn(minecraft::listener(25565));
    let smtp_listener = tokio::spawn(smtp::listener(1025));

    let _ = http_listener.await.expect("http_listener failed");
    _ = https_listener.await.expect("https_listener failed");
    _ = minecraft_listener.await.expect("minecraft_listener failed");
    _ = smtp_listener.await.expect("smtp_listener failed");

    Ok(())
}
