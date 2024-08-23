mod services;
mod utils;
use env_logger::Builder;
use log::LevelFilter;
use services::http;
use services::https;
use services::quic;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    Builder::new().filter(None, LevelFilter::Info).init();

    let http_listener = tokio::spawn(http::listener(80));
    let https_listener = tokio::spawn(https::listener(443));
    let quic_listener = tokio::spawn(quic::listener(443));

    let _ = http_listener.await.expect("http_listener failed");
    _ = https_listener.await.expect("https_listener failed");
    _ = quic_listener.await.expect("quic_listener failed");

    Ok(())
}
