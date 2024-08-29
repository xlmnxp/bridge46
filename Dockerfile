# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /app

# Copy the source code to the container
COPY . ./

# Build the application
RUN cargo build --release

# Expose ports 80, 443 and 25565
EXPOSE 80 443 25565

# Fix colors
ENV TERM=xterm-256color

# Run the application
CMD ["/app/target/release/bridge46"]