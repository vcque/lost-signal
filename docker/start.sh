#!/bin/sh
set -e

echo "Starting Lost Signal game server and nginx..."

# Check if SSL certificates exist
CERT_PATH="/etc/letsencrypt/live/losig.vcque.eu"
if [ ! -f "$CERT_PATH/fullchain.pem" ]; then
    echo "No SSL certificates found at $CERT_PATH"
    echo "Generating self-signed certificate for initial startup..."

    # Create directory structure
    mkdir -p "$CERT_PATH"

    # Generate self-signed certificate
    openssl req -x509 -nodes -newkey rsa:2048 \
        -keyout "$CERT_PATH/privkey.pem" \
        -out "$CERT_PATH/fullchain.pem" \
        -days 1 \
        -subj "/CN=losig.vcque.eu"

    echo "Self-signed certificate generated (valid for 1 day)"
    echo "IMPORTANT: Replace with Let's Encrypt certificate using init-ssl.sh"
fi

# Start supervisord to manage all services
exec /usr/bin/supervisord -c /etc/supervisord.conf