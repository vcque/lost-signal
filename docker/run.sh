#!/bin/bash
set -e

echo "Running Lost Signal container on port 8080..."
docker run --rm -p 8080:8080 --name lost-signal-game lost-signal