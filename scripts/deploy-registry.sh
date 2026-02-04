#!/bin/bash
# Quick deployment script for Vais Registry Server
# Usage: ./scripts/deploy-registry.sh [admin-password]

set -e

ADMIN_PASSWORD="${1:-changeme}"
REGISTRY_PORT="${REGISTRY_PORT:-3000}"

echo "=========================================="
echo "Vais Registry Deployment Script"
echo "=========================================="
echo ""

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Error: Docker is not installed"
    echo "   Please install Docker from https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if Docker daemon is running
if ! docker info &> /dev/null; then
    echo "❌ Error: Docker daemon is not running"
    echo "   Please start Docker and try again"
    exit 1
fi

echo "✓ Docker is available"
echo ""

# Build the registry image
echo "📦 Building registry image..."
docker build -f Dockerfile.registry -t vais-registry:latest . || {
    echo "❌ Build failed"
    exit 1
}
echo "✓ Image built successfully"
echo ""

# Stop and remove existing container if it exists
if docker ps -a --format '{{.Names}}' | grep -q "^vais-registry$"; then
    echo "🛑 Stopping existing container..."
    docker stop vais-registry || true
    docker rm vais-registry || true
fi

# Run the registry container
echo "🚀 Starting registry server..."
docker run -d \
    --name vais-registry \
    -p "${REGISTRY_PORT}:3000" \
    -v vais-registry-data:/data \
    -e VAIS_REGISTRY_ADMIN_USER=admin \
    -e VAIS_REGISTRY_ADMIN_PASS="${ADMIN_PASSWORD}" \
    -e RUST_LOG=vais_registry_server=info \
    --restart unless-stopped \
    vais-registry:latest

# Wait for registry to be healthy
echo ""
echo "⏳ Waiting for registry to start..."
for i in {1..30}; do
    if curl -s -o /dev/null -w "%{http_code}" "http://localhost:${REGISTRY_PORT}/health" | grep -q "200"; then
        echo "✓ Registry is healthy!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Registry failed to start within 30 seconds"
        echo "   Check logs with: docker logs vais-registry"
        exit 1
    fi
    sleep 1
done

echo ""
echo "=========================================="
echo "✓ Registry deployed successfully!"
echo "=========================================="
echo ""
echo "Registry URL:    http://localhost:${REGISTRY_PORT}"
echo "Admin Username:  admin"
echo "Admin Password:  ${ADMIN_PASSWORD}"
echo ""
echo "Next steps:"
echo "  1. Login to the registry:"
echo "     vaisc pkg login --registry http://localhost:${REGISTRY_PORT}"
echo ""
echo "  2. Publish a package:"
echo "     cd your-package-dir"
echo "     vaisc pkg publish --registry http://localhost:${REGISTRY_PORT}"
echo ""
echo "  3. Search packages:"
echo "     vaisc pkg search <query> --registry http://localhost:${REGISTRY_PORT}"
echo ""
echo "Useful commands:"
echo "  - View logs:    docker logs -f vais-registry"
echo "  - Stop:         docker stop vais-registry"
echo "  - Restart:      docker restart vais-registry"
echo "  - Remove:       docker rm -f vais-registry"
echo "  - Backup data:  docker run --rm -v vais-registry-data:/data -v \$(pwd):/backup alpine tar czf /backup/registry-backup.tar.gz /data"
echo ""
