# Bridge46
Bridge46 is simple bridge between IPv4 and IPv6
it's works by listen on IPv4 well known services and forward the traffic to IPv6 services

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
3. Add A record of IPv4 of the server to your DNS records (make sure you have IPv6 AAAA record too)

## Supported services
- HTTP
- HTTPS
- Websocket (ws) and Secure Websocket (wss)

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details
