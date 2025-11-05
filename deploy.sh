#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$(realpath "$0")")"

# Tag name for Docker Hub
IMAGE_NAME="signet-broker"

# Optional version tag (use Git or date)
VERSION_TAG="${1:-latest}"

# Build the image
echo "Building ${IMAGE_NAME}:${VERSION_TAG} image..."
docker build -t "${IMAGE_NAME}:${VERSION_TAG}" .

echo "💾 Saving Docker image to broker_image.tar..."
docker save "${IMAGE_NAME}:${VERSION_TAG}" | ssh frog 'docker load'

DOCKER_HOST="ssh://frog" docker compose up --build

echo "✅ Deployment complete!"
