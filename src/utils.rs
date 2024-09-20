use hickory_client::client::{AsyncClient, ClientHandle};
use hickory_client::op::DnsResponse;
use hickory_client::proto::iocompat::AsyncIoTokioAsStd;
use hickory_proto::rr::{DNSClass, Name, RData, Record, RecordType};
use hickory_proto::tcp::TcpClientStream;
use std::io::Error;
use std::net::IpAddr;
use std::str::FromStr;
use tokio::net::TcpStream;

pub async fn resolve_addr(addr: &str) -> std::io::Result<IpAddr> {
    let (stream, sender) = TcpClientStream::<AsyncIoTokioAsStd<TcpStream>>::new(
        get_dns_server()
            .parse()
            .expect("Invalid DNS server address"),
    );
    let dns_client = AsyncClient::new(stream, sender, None);
    let (mut dns_client, bg) = dns_client.await.expect("dns connection failed");

    // make sure to run the background task
    tokio::spawn(bg);

    let response_ipv6: DnsResponse = dns_client
        .query(Name::from_str(addr)?, DNSClass::IN, RecordType::AAAA)
        .await
        .expect("Failed to query");

    let answers_ipv6: &[Record] = response_ipv6.answers();

    if answers_ipv6.len() == 0 {
        return Err(Error::new(
            std::io::ErrorKind::Other,
            "No AAAA records found",
        ));
    }

    // check if DNS has A/AAAA record pointed to Bridge46 IPv4/IPv6 address
    let bridge_ipv4 = get_bridge46_ipv4();
    let bridge_ipv6 = get_bridge46_ipv6();
    if bridge_ipv4 != "" || bridge_ipv6 != "" {
        let response_ipv4: DnsResponse = dns_client
            .query(Name::from_str(addr)?, DNSClass::IN, RecordType::A)
            .await
            .expect("Failed to query");

        let answers_ipv4: &[Record] = response_ipv4.answers();

        if ![answers_ipv4, answers_ipv6].concat().iter().any(|answer| {
            if let Some(RData::A(ref ip)) = answer.data() {
                if ip.to_string() == bridge_ipv4 {
                    return true;
                }
            } else if let Some(RData::AAAA(ref ip)) = answer.data() {
                if ip.to_string() == bridge_ipv6 {
                    return true;
                }
            }
            false
        }) {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "No A/AAAA record points to IPv4/IPv6 of Bridge46 service",
            ));
        }
    }

    log::info!(
        "DNS Resolver: {} Has AAAA Records: {}",
        addr,
        answers_ipv6
            .iter()
            .map(|r| r
                .data()
                .expect("Cannot process IP Data")
                .ip_addr()
                .expect("Invalid IP address")
                .to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    for answer in answers_ipv6 {
        if let Some(RData::AAAA(ref ip)) = answer.data() {
            let bridge_ipv6 = get_bridge46_ipv6();
            if bridge_ipv6 != "" && ip.to_string() == bridge_ipv6 {
                log::info!(
                    "DNS Resolver: {} requested IPv6 is same as Bridge46 service IPv6",
                    addr
                );
                continue;
            }

            return Ok(ip
                .to_string()
                .parse::<IpAddr>()
                .expect("Invalid IP address"));
        } else {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "No AAAA records found",
            ));
        }
    }

    Err(Error::new(
        std::io::ErrorKind::Other,
        "No AAAA records found",
    ))
}

pub async fn forward(source: TcpStream, distance: String) -> Option<()> {
    let source_addr = source.peer_addr().ok()?;
    let server: Result<TcpStream, tokio::io::Error> = TcpStream::connect(distance.clone()).await;
    if server.is_err() {
        log::error!(
            "{} Failed to connect to upstream: {}",
            source_addr,
            distance
        );
        return None;
    }

    let server: TcpStream = server.ok()?;
    let (mut eread, mut ewrite) = source.into_split();
    let (mut oread, mut owrite) = server.into_split();
    log::info!("{} Connected to upstream: {}", source_addr, distance);
    tokio::spawn(async move { tokio::io::copy(&mut eread, &mut owrite).await });
    tokio::spawn(async move { tokio::io::copy(&mut oread, &mut ewrite).await });

    Some(())
}

pub fn get_dns_server() -> String {
    std::env::var("DNS_SERVER").unwrap_or_else(|_| "1.1.1.1:53".into())
}

pub fn get_bind_address() -> String {
    std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "[::]".into())
}

pub fn get_bridge46_ipv4() -> String {
    std::env::var("BRIDGE46_IPV4").unwrap_or_else(|_| "".into())
}

pub fn get_bridge46_ipv6() -> String {
    std::env::var("BRIDGE46_IPV6").unwrap_or_else(|_| "".into())
}
