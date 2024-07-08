use tls_parser::{parse_tls_extensions, parse_tls_plaintext};
use hickory_client::client::{AsyncClient, ClientHandle};
use hickory_client::proto::iocompat::AsyncIoTokioAsStd;
use hickory_client::op::DnsResponse;
use hickory_proto::rr::{DNSClass, Name, RData, Record, RecordType};
use hickory_proto::tcp::TcpClientStream;
use std::io::Error;
use std::net::IpAddr;
use tokio::net::TcpStream;

use crate::DNS_SERVER;

pub fn get_sni_from_packet(packet: &[u8]) -> Option<String> {
    let res: Result<
        (&[u8], tls_parser::TlsPlaintext),
        tls_parser::Err<tls_parser::nom::error::Error<&[u8]>>,
    > = parse_tls_plaintext(&packet);
    if res.is_err() {
        return None;
    }
    let tls_message: &tls_parser::TlsMessage = &res.unwrap().1.msg[0];
    if let tls_parser::TlsMessage::Handshake(handshake) = tls_message {
        if let tls_parser::TlsMessageHandshake::ClientHello(client_hello) = handshake {
            // get the extensions
            let extensions: &[u8] = client_hello.ext.unwrap();
            // parse the extensions
            let res: Result<
                (&[u8], Vec<tls_parser::TlsExtension>),
                tls_parser::Err<tls_parser::nom::error::Error<&[u8]>>,
            > = parse_tls_extensions(extensions);
            // iterate over the extensions and find the SNI
            for extension in res.unwrap().1 {
                if let tls_parser::TlsExtension::SNI(sni) = extension {
                    // get the hostname
                    let hostname: &[u8] = sni[0].1;
                    let s: String = match String::from_utf8(hostname.to_vec()) {
                        Ok(v) => v,
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };
                    return Some(s);
                }
            }
        }
    }
    None
}

pub async fn resolve_addr(addr: &str) -> std::io::Result<IpAddr>{
    let (stream, sender) = TcpClientStream::<AsyncIoTokioAsStd<TcpStream>>::new(DNS_SERVER.parse().expect("Invalid DNS server address"));
    let dns_client = AsyncClient::new(stream, sender, None);
    let (mut dns_client, bg) = dns_client.await.expect("dns connection failed");

    // make sure to run the background task
    tokio::spawn(bg);
    
    let response: DnsResponse = dns_client
        .query(Name::from_utf8(addr).unwrap(), DNSClass::IN, RecordType::AAAA)
        .await
        .expect("Failed to query");
    let answers: &[Record] = response.answers();

    if answers.len() == 0 {
        return Err(Error::new(std::io::ErrorKind::Other, "No AAAA records found"));
    }

    log::info!("DNS Resolver: {} Has AAAA Records: {}", addr, answers.iter().map(|r| r.data().expect("Cannot process IP Data").ip_addr().expect("Invalid IP address").to_string()).collect::<Vec<String>>().join(", "));

    if let Some(RData::AAAA(ref ip)) = answers[0].data() {
        return Ok(ip.to_string().parse::<IpAddr>().expect("Invalid IP address"));
    } else {
        return Err(Error::new(std::io::ErrorKind::Other, "No AAAA records found"));
    }
}