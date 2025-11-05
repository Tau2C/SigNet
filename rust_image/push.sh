#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$(realpath "$0")")"

# Tag name for Docker Hub
IMAGE_NAME="xblackxsnakex/tau2c:rust-musl-builder"

# Optional version tag (use Git or date)
VERSION_TAG="${1:-latest}"  # Example: ./publish.sh v1.0.0

# Build the image
echo "🦀 Building Rust MUSL builder image..."
docker build -t "${IMAGE_NAME}_${VERSION_TAG}" .

# Push the image
echo "📦 Pushing ${IMAGE_NAME}_${VERSION_TAG} to Docker Hub..."
docker push "${IMAGE_NAME}_${VERSION_TAG}"

echo "✅ Done!"
