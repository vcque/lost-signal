#!/bin/bash
set -e

echo "Running Lost Signal container on ports 80 and 443..."
docker run -d \
  --name lost-signal \
  --restart=unless-stopped \
  -p 80:80 \
  -p 443:443 \
  -v $(pwd)/letsencrypt:/etc/letsencrypt \
  -v $(pwd)/letsencrypt-lib:/var/lib/letsencrypt \
  lost-signal

echo "Container started: lost-signal"
echo "View logs with: docker logs -f lost-signal"