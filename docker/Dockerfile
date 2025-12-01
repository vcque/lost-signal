# Multi-stage build for WASM client
FROM rust:1.91-alpine as wasm-builder
RUN apk add --no-cache musl-dev
RUN cargo install trunk --version 0.21.14 --locked
RUN rustup target add wasm32-unknown-unknown
WORKDIR /app
COPY . .
RUN cd crates/client-wasm && trunk build --release --public-url /

# Build Rust server (headless mode)
FROM rust:1.91-alpine as server-builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY . .
RUN cargo build --release -p losig-server --no-default-features

# Runtime image with nginx (Alpine)
FROM nginx:1.27-alpine
RUN apk add --no-cache supervisor

# Copy built assets
COPY --from=wasm-builder /app/crates/client-wasm/dist /usr/share/nginx/html
COPY --from=server-builder /app/target/release/losig-server /usr/local/bin/losig-server

# Copy nginx configuration
COPY docker/nginx.conf /etc/nginx/nginx.conf

# Copy supervisor configuration
COPY docker/supervisord.conf /etc/supervisord.conf

# Copy startup script
COPY docker/start.sh /start.sh
RUN chmod +x /start.sh

EXPOSE 8080
CMD ["/start.sh"]
