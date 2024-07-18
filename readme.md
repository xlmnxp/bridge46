# Bridge46
Bridge46 is simple bridge between IPv4 and IPv6
it's works by listen on IPv4 well known services and forward the traffic to IPv6 services

## Supported services
- HTTP
- HTTPS
- Websocket (ws) and Secure Websocket (wss)

## How to use
### Build from source
1. Clone the repository
2. Run the following command to build and run the project
```bash
cargo run
```
3. Add A record of IPv4 of the server to your DNS records (make sure you have IPv6 AAAA record too)
4. Now you can access your IPv6 services using IPv4

### Docker
1. Pull the image from docker hub (optional because the image will be pulled automatically when you run the container)
```bash
docker pull xlmnxp/bridge46:latest
```
2. Run the image
```bash
docker run -d -p 80:80 -p 443:443 --name bridge46 xlmnxp/bridge46:latest
```

Optionally you can pass the following environment variables to the container
| Environment Variable | Description | Default Value |
|----------------------|-------------|---------------|
| DNS_SERVER | specify the DNS server to use | `1.1.1.1:53` |
| BIND_ADDRESS | specify the address to bind to | `::` |
| BRIDGE46_IPV4 | specify the IPv4 address for service validation of A Records point to the service (see #1) | empty |
| BRIDGE46_IPV6 | specify the IPv4 address for service validation of AAAA Records point to the service (see #1) | empty |

3. Add A record of IPv4 of the server to your DNS records (make sure you have IPv6 AAAA record too)

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details
