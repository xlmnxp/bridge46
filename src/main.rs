mod utils;
mod services;
use env_logger::Builder;
use log::LevelFilter;
use services::http;
use services::https;

pub const DNS_SERVER: &str = "1.1.1.1:53";
pub const BIND_ADDRESS: &str = "::";


#[tokio::main]
async fn main() -> std::io::Result<()> {
    Builder::new()
    .filter(None, LevelFilter::Info)
    .init();

    let http_listener = tokio::spawn(http::listener(BIND_ADDRESS, 80));
    let https_listener = tokio::spawn(https::listener(BIND_ADDRESS, 443));

    let _ = http_listener.await.expect("https_listener failed");
    _ = https_listener.await.expect("https_listener failed");
    Ok(())
}
