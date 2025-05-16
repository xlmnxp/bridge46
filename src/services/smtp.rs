use std::sync::Arc;

use crate::utils::{forward, get_bind_address, resolve_addr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use futures::lock::Mutex;

struct Mail {
    sender: String,
    recipient: String,
    data: String,
}

impl Mail {
    async fn from(client: Arc<Mutex<TcpStream>>) -> Self {
        let mut mail_sender = String::new();
        let mut mail_recipient = String::new();
        let mut mail_data = String::new();

        client
            .try_lock()
            .expect("Cannot lock client stream")
            .write(b"220 bridge46 ESMTP Postfix\r\n")
            .await
            .ok()
            .expect("write failed");

        loop {
            let mut buf: Vec<u8> = vec![0; 2048];
            let read_size = client
                .try_lock()
                .expect("Cannot lock client stream")
                .read(&mut buf)
                .await
                .ok()
                .expect("write failed");
            let client_request = String::from_utf8_lossy(&buf[..read_size]);

            match client_request.clone() {
                s if s.starts_with("HELO") => {
                    client
                        .try_lock()
                        .expect("Cannot lock client stream")
                        .write(b"250 Hello\r\n")
                        .await
                        .ok()
                        .expect("write failed");
                }
                s if s.starts_with("EHLO") => {
                    client
                        .try_lock()
                        .expect("Cannot lock client stream")
                        .write(b"250 Hello\r\n")
                        .await
                        .ok()
                        .expect("write failed");
                }
                s if s.starts_with("MAIL FROM") => {
                    mail_sender = client_request
                        .split("MAIL FROM:")
                        .last()
                        .expect("No sender found")
                        .trim_end_matches("\r\n")
                        .to_string();
                    mail_sender = mail_sender[1..mail_sender.len() - 1].to_string();

                    client
                        .try_lock()
                        .expect("Cannot lock client stream")
                        .write(b"250 Ok\r\n")
                        .await
                        .ok()
                        .expect("write failed");
                }
                s if s.starts_with("RCPT TO") => {
                    mail_recipient = client_request
                        .split("RCPT TO:")
                        .last()
                        .expect("No recipient found")
                        .trim_end_matches("\r\n")
                        .to_string();
                    mail_recipient = mail_recipient[1..mail_recipient.len() - 1].to_string();

                    client
                        .try_lock()
                        .expect("Cannot lock client stream")
                        .write(b"250 Ok\r\n")
                        .await
                        .ok()
                        .expect("write failed");
                }
                s if s.eq("DATA\r\n") => {
                    client
                        .try_lock()
                        .expect("Cannot lock client stream")
                        .write(b"354 End data with <CR><LF>.<CR><LF>\r\n")
                        .await
                        .ok()
                        .expect("write failed");

                    loop {
                        let mut buf: Vec<u8> = vec![0; 2048];
                        let read_size = client
                            .try_lock()
                            .expect("Cannot lock client stream")
                            .read(&mut buf)
                            .await
                            .ok()
                            .expect("write failed");
                        let client_request = String::from_utf8_lossy(&buf[..read_size]);

                        mail_data.push_str(&client_request);
                        if mail_data.ends_with("\r\n.\r\n") {
                            println!("it ends with \\r\\n.\\r\\n");
                            break;
                        }
                    }

                    client
                        .try_lock()
                        .expect("Cannot lock client stream")
                        .write(b"250 Ok\r\n")
                        .await
                        .ok()
                        .expect("write failed");
                }
                s if s.eq("QUIT\r\n") => {
                    client
                        .try_lock()
                        .expect("Cannot lock client stream")
                        .write(b"221 Bye\r\n")
                        .await
                        .ok()
                        .expect("write failed");
                    break;
                }
                _ => {
                    client
                        .try_lock()
                        .expect("Cannot lock client stream")
                        .write(b"500 Error\r\n")
                        .await
                        .ok()
                        .expect("write failed");
                }
            }
        }

        Self {
            sender: mail_sender,
            recipient: mail_recipient,
            data: mail_data,
        }
    }
}

async fn handle_connection(client: TcpStream, port: u16) -> Option<()> {
    let src_addr = client.peer_addr().ok()?;

    // store TCPStream into
    let client = Arc::new(Mutex::new(client));

    let Mail {
        sender: mail_sender,
        recipient: mail_recipient,
        data: mail_data,
    } = Mail::from(client.clone()).await;

    println!("Mail Sender: {:?}", mail_sender);
    println!("Mail Reception: {:?}", mail_recipient);
    println!("Mail Data: {:?}", mail_data);

    let host = mail_recipient.split('@').last().map(|s| s.to_string());

    println!("Host: {:?}", host);

    loop {
        if let Some(host_string) = host.clone() {
            match resolve_addr(&host_string).await {
                Ok(ip) => {
                    let distance = format!("[{}]:{}", ip, port);
                    
                    ;
                }
                Err(_) => {
                    log::error!(
                        "HTTP {} No AAAA records found for {}",
                        src_addr,
                        host_string
                    );
                }
            }
        }
        break;
    }
    None
}

pub async fn send_email(
    distention: String,
    sender: String,
    subject: String,
    body: String,
) {
    

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
