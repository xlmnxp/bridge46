use tokio::net::{TcpListener, TcpStream};
use crate::utils::{forward, get_bind_address, resolve_addr};

struct MinecraftServer {
    hostname: String,
    port: u16,
    protocol_version: i32,
}

impl MinecraftServer {
    fn read_server_info(packet: &[u8]) -> Result<MinecraftServer, std::io::Error> {
        // Read packet ID
        if packet[1] != 0x00 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unexpected packet ID",
            ));
        }

        // Read protocol version
        let protocol_version: i32 = Self::read_var_int(&packet[2..4])?;

        // Read hostname length
        let hostname_length = packet[4];

        println!("{:?}", hostname_length);

        // Read hostname
        let hostname =
            String::from_utf8_lossy(&packet[5..5 + hostname_length as usize]).to_string();

        // Read port
        let port = (packet[5 + hostname_length as usize] as u16) << 8 | packet[6 + hostname_length as usize] as u16;

        Ok(MinecraftServer {
            hostname,
            port,
            protocol_version,
        })
    }

    fn read_var_int(mut packet: &[u8]) -> Result<i32, std::io::Error> {
        let mut result: i32 = 0;
        let mut position: i32 = 0;

        println!("{:?}", packet);

        loop {
            if position > 35 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "VarInt is too big",
                ));
            }

            let byte = packet[0] as i32;
            packet = &packet[1..];

            result |= (byte & 0b0111_1111) << position;
            position += 7;

            if (byte & 0b1000_0000) == 0 {
                break;
            }
        }

        Ok(result)
    }
}

// implement debug for MinecraftServer
impl std::fmt::Debug for MinecraftServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MinecraftServer")
            .field("hostname", &self.hostname)
            .field("port", &self.port)
            .field("protocol_version", &self.protocol_version)
            .finish()
    }
}

async fn handle_connection(client: TcpStream, _port: u16) -> Option<()> {
    let src_addr = client.peer_addr().ok()?;

    // read request header and get the host
    let mut buf: Vec<u8> = vec![0; 256];
    client.peek(&mut buf).await.expect("peek failed");

    let server_info = MinecraftServer::read_server_info(&buf).expect("Failed to read server info");

    let ip = resolve_addr(&server_info.hostname).await.expect("Failed to resolve hostname");
    let distance = format!("[{}]:{}", ip, server_info.port);

    log::info!(
        "Minecraft {} Choose AAAA record for {}: {}",
        src_addr,
        server_info.hostname,
        distance
    );

    let distance = format!("[{}]:{}", ip, server_info.port);

    forward(client, distance).await
}

pub async fn listener(port: u16) -> std::io::Result<()> {
    let listener: TcpListener =
        TcpListener::bind(format!("{}:{}", get_bind_address(), port)).await?;
    log::info!("Listening on {}", listener.local_addr()?);

    loop {
        let (client, _) = listener.accept().await?;
        tokio::spawn(async move {
            handle_connection(client, port).await;
        });
    }
}
