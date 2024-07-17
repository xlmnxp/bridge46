use hickory_client::client::{AsyncClient, ClientHandle};
use hickory_client::op::DnsResponse;
use hickory_client::proto::iocompat::AsyncIoTokioAsStd;
use hickory_proto::rr::{DNSClass, Name, RData, Record, RecordType};
use hickory_proto::tcp::TcpClientStream;
use std::io::Error;
use std::net::IpAddr;
use std::str::FromStr;
use tokio::net::TcpStream;

use crate::DNS_SERVER;

pub async fn resolve_addr(addr: &str) -> std::io::Result<IpAddr> {
    let (stream, sender) = TcpClientStream::<AsyncIoTokioAsStd<TcpStream>>::new(
        DNS_SERVER.parse().expect("Invalid DNS server address"),
    );
    let dns_client = AsyncClient::new(stream, sender, None);
    let (mut dns_client, bg) = dns_client.await.expect("dns connection failed");

    // make sure to run the background task
    tokio::spawn(bg);

    let response: DnsResponse = dns_client
        .query(Name::from_str(addr)?, DNSClass::IN, RecordType::AAAA)
        .await
        .expect("Failed to query");
    let answers: &[Record] = response.answers();

    if answers.len() == 0 {
        return Err(Error::new(
            std::io::ErrorKind::Other,
            "No AAAA records found",
        ));
    }

    log::info!(
        "DNS Resolver: {} Has AAAA Records: {}",
        addr,
        answers
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

    if let Some(RData::AAAA(ref ip)) = answers[0].data() {
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
