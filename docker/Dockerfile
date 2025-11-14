# Multi-stage build for WASM client
FROM rust:1.91-bullseye as wasm-builder
RUN cargo install trunk --version 0.21.14 --locked
RUN rustup target add wasm32-unknown-unknown
WORKDIR /app
COPY . .
RUN cd crates/client-wasm && trunk build --release --public-url /

# Build Rust server
FROM rust:1.91-bullseye as server-builder
WORKDIR /app
COPY . .
RUN cargo build --release -p losig-server

# Runtime image with nginx
FROM nginx:1.25-bullseye
RUN apt-get update && apt-get install -y supervisor util-linux && rm -rf /var/lib/apt/lists/*

# Copy built assets
COPY --from=wasm-builder /app/crates/client-wasm/dist /usr/share/nginx/html
COPY --from=server-builder /app/target/release/losig-server /usr/local/bin/losig-server

# Copy nginx configuration
COPY docker/nginx.conf /etc/nginx/nginx.conf

# Copy supervisor configuration
COPY docker/supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Copy startup script
COPY docker/start.sh /start.sh
RUN chmod +x /start.sh

EXPOSE 8080
CMD ["/start.sh"]
