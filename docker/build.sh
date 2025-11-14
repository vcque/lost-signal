#!/bin/bash
set -e

echo "Building Lost Signal Docker image..."

# Build from project root using docker subdirectory
cd "$(dirname "$0")/.."
docker build -f docker/Dockerfile -t lost-signal .

echo "Build complete! Run with:"
echo "  docker run -p 8080:8080 lost-signal"