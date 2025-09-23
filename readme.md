# Bridge46
Bridge46 is simple bridge between IPv4 and IPv6
it's works by listen on IPv4 well known services and forward the traffic to IPv6 services

## Supported services
- HTTP (configurable ports, default: 80)
- HTTPS (configurable ports, default: 443)
- Websocket (ws) and Secure Websocket (wss) over HTTP/HTTPS
- Minecraft (TCP) (configurable port, default: 25565)

## How to use
### Build from source
1. Clone the repository
2. Run the following command to build and run the project
```bash
cargo run
```

### Port Configuration
Bridge46 supports configurable ports via command line arguments:

```bash
# Default ports (80, 443, 25565)
cargo run

# Custom single ports
cargo run -- --http-ports 8080 --https-ports 8443 --minecraft-ports 25566

# Port ranges (supports ranges like 80-90)
cargo run -- --http-ports 80-85 --https-ports 443-450

# Comma-separated ports
cargo run -- --http-ports 80,8080,9000 --https-ports 443,8443,9443 --minecraft-ports 25565,25566

# Mixed single ports and ranges (space-separated)
cargo run -- --http-ports 80 8080-8082 --https-ports 443 8443-8445

# Mixed single ports and ranges (comma-separated)
cargo run -- --http-ports 80,8080-8082,9000 --https-ports 443,8443-8445,9443 --minecraft-ports 25565,25570-25572

# Multiple HTTP/HTTPS ports (space-separated)
cargo run -- --http-ports 80 8080 --https-ports 443 8443
```

#### CLI Options
- `--http-ports`: HTTP ports to listen on (supports ranges and comma-separated values) [default: 80]
- `--https-ports`: HTTPS ports to listen on (supports ranges and comma-separated values) [default: 443]
- `--minecraft-ports`: Minecraft ports to listen on (supports ranges and comma-separated values) [default: 25565]

#### Port Specification Examples
**Single ports:**
- `8080`: Creates listener on port 8080
- `80,8080,9000`: Creates listeners on ports 80, 8080, 9000

**Port ranges:**
- `80-85`: Creates listeners on ports 80, 81, 82, 83, 84, 85
- `443-450`: Creates listeners on ports 443, 444, 445, 446, 447, 448, 449, 450
- `8080-8082`: Creates listeners on ports 8080, 8081, 8082

**Mixed single ports and ranges:**
- `80,8080-8082,9000`: Creates listeners on ports 80, 8080, 8081, 8082, 9000
- `443,8443-8445,9443`: Creates listeners on ports 443, 8443, 8444, 8445, 9443
- `25565,25570-25572`: Creates listeners on ports 25565, 25570, 25571, 25572

**Note**: Port ranges are limited to a maximum of 1000 ports to prevent resource exhaustion.

#### Getting Help
To see all available CLI options:
```bash
cargo run -- --help
```

#### Command Usage
```
Usage: bridge46 [OPTIONS]

Options:
      --http-ports <HTTP_PORTS>
          HTTP ports to listen on (supports ranges like 80-90) [default: 80]
      --https-ports <HTTPS_PORTS>
          HTTPS ports to listen on (supports ranges like 443-450) [default: 443]
      --minecraft-ports <MINECRAFT_PORTS>
          Minecraft ports to listen on (supports ranges and comma-separated values) [default: 25565]
  -h, --help
          Print help
  -V, --version
          Print version
```

3. Add A record of IPv4 of the server to your DNS records (make sure you have IPv6 AAAA record too)
4. Now you can access your IPv6 services using IPv4

### Docker
1. Pull the image from docker hub (optional because the image will be pulled automatically when you run the container)
```bash
docker pull xlmnxp/bridge46:latest
```
2. Run the image with default ports
```bash
docker run -d -p 80:80 -p 443:443 -p 25565:25565 --name bridge46 xlmnxp/bridge46:latest
```

3. Run with custom ports
```bash
# Custom single ports
docker run -d -p 8080:8080 -p 8443:8443 -p 25566:25566 --name bridge46 xlmnxp/bridge46:latest --http-ports 8080 --https-ports 8443 --minecraft-ports 25566

# Port ranges
docker run -d -p 80-85:80-85 -p 443-445:443-445 -p 25565-25567:25565-25567 --name bridge46 xlmnxp/bridge46:latest --http-ports 80-85 --https-ports 443-445 --minecraft-ports 25565-25567

# Comma-separated ports
docker run -d -p 80:80 -p 8080:8080 -p 443:443 -p 8443:8443 -p 25565:25565 -p 25566:25566 --name bridge46 xlmnxp/bridge46:latest --http-ports 80,8080 --https-ports 443,8443 --minecraft-ports 25565,25566
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
