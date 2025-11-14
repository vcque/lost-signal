#!/bin/sh
set -e

echo "Starting Lost Signal game server and nginx..."

# Start supervisord to manage both services
exec /usr/bin/supervisord -c /etc/supervisor/conf.d/supervisord.conf