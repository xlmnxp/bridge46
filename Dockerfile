# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /app

# Copy the source code to the container
COPY . ./

# Build the application
RUN cargo build --release

# Expose ports 443 and 80
EXPOSE 443 80

# Run the application
CMD ["/app/target/release/bridge46"]