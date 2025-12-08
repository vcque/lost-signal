#!/bin/bash
# Usage: ./docker/init-ssl.sh [--staging]

DOMAIN="losig.vcque.eu"
EMAIL="admin@vcque.eu"  # Update with your email
STAGING=""

if [ "$1" = "--staging" ]; then
    STAGING="--staging"
    echo "Using Let's Encrypt STAGING environment (test certificates)"
fi

# Create directory if it doesn't exist
mkdir -p letsencrypt

# Stop any service using port 80 first!
docker run --rm \
  -v $(pwd)/letsencrypt:/etc/letsencrypt \
  -v $(pwd)/letsencrypt-lib:/var/lib/letsencrypt \
  -p 80:80 \
  certbot/certbot certonly \
    --standalone \
    -d $DOMAIN \
    --email $EMAIL \
    --agree-tos \
    --non-interactive \
    $STAGING

echo ""
echo "✓ Certificates obtained!"
echo "  Location: $(pwd)/letsencrypt/live/$DOMAIN/"
echo ""
echo "Now run: ./docker/build.sh && ./docker/run.sh"
