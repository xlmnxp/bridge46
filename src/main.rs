mod services;
mod utils;

use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use services::http;
use services::https;
use services::minecraft;

/// Parse a port specification which can be either a single port or a range (e.g., "80" or "80-90")
fn parse_port_spec(spec: &str) -> Result<Vec<u16>, String> {
    if spec.contains('-') {
        let parts: Vec<&str> = spec.split('-').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid port range format: {}", spec));
        }
        
        let start: u16 = parts[0].parse()
            .map_err(|_| format!("Invalid start port: {}", parts[0]))?;
        let end: u16 = parts[1].parse()
            .map_err(|_| format!("Invalid end port: {}", parts[1]))?;
            
        if start > end {
            return Err(format!("Start port {} cannot be greater than end port {}", start, end));
        }

        Ok((start..=end).collect())
    } else {
        let port: u16 = spec.parse()
            .map_err(|_| format!("Invalid port number: {}", spec))?;
        Ok(vec![port])
    }
}

/// Parse a single port specification (single port, range, or comma-separated values)
fn parse_port_single(spec: &str) -> Result<Vec<u16>, String> {
    if spec.contains(',') {
        // Handle comma-separated values
        let mut ports = Vec::new();
        for part in spec.split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            let mut parsed_ports = parse_port_spec(trimmed)?;
            ports.append(&mut parsed_ports);
        }
        Ok(ports)
    } else {
        // Handle single port or range
        parse_port_spec(spec)
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// HTTP ports to listen on (supports ranges like 80-90)
    #[arg(long, default_values = ["80"], value_parser = parse_port_single)]
    http_ports: Vec<Vec<u16>>,

    /// HTTPS ports to listen on (supports ranges like 443-450)
    #[arg(long, default_values = ["443"], value_parser = parse_port_single)]
    https_ports: Vec<Vec<u16>>,

    /// Minecraft ports to listen on (supports ranges and comma-separated values)
    #[arg(long, default_values = ["25565"], value_parser = parse_port_single)]
    minecraft_ports: Vec<Vec<u16>>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    Builder::new().filter(None, LevelFilter::Info).init();

    let args = Args::parse();

    // Flatten HTTP ports from nested vectors
    let http_ports: Vec<u16> = args.http_ports.into_iter().flatten().collect();
    for http_port in http_ports {
        tokio::spawn(http::listener(http_port));
    }

    // Flatten HTTPS ports from nested vectors
    let https_ports: Vec<u16> = args.https_ports.into_iter().flatten().collect();
    for https_port in https_ports {
        tokio::spawn(https::listener(https_port));
    }

    // Flatten Minecraft ports from nested vectors
    let minecraft_ports: Vec<u16> = args.minecraft_ports.into_iter().flatten().collect();
    for minecraft_port in minecraft_ports {
        tokio::spawn(minecraft::listener(minecraft_port));
    }

    // Wait forever
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
