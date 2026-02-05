#!/bin/bash
# Vais Package Registry - Fly.io Deployment Script
# Usage: ./scripts/fly-deploy.sh [command]
# Commands:
#   setup   - Initial setup (create app, volume, secrets)
#   deploy  - Deploy the registry
#   logs    - View logs
#   status  - Check status
#   ssh     - SSH into the running machine

set -e

APP_NAME="vais-registry"
REGION="nrt"  # Tokyo
VOLUME_NAME="vais_registry_data"
VOLUME_SIZE="1"  # GB

cd "$(dirname "$0")/.."

case "${1:-deploy}" in
  setup)
    echo "Setting up Fly.io for Vais Registry..."

    # Check if flyctl is installed
    if ! command -v flyctl &> /dev/null; then
      echo "Error: flyctl is not installed. Install from https://fly.io/docs/getting-started/installing-flyctl/"
      exit 1
    fi

    # Check login
    if ! flyctl auth whoami &> /dev/null; then
      echo "Please login to Fly.io first:"
      flyctl auth login
    fi

    # Create app if not exists
    if ! flyctl apps list | grep -q "$APP_NAME"; then
      echo "Creating app: $APP_NAME"
      flyctl apps create "$APP_NAME" --org personal
    else
      echo "App $APP_NAME already exists"
    fi

    # Create volume if not exists
    if ! flyctl volumes list --app "$APP_NAME" 2>/dev/null | grep -q "$VOLUME_NAME"; then
      echo "Creating volume: $VOLUME_NAME ($VOLUME_SIZE GB) in $REGION"
      flyctl volumes create "$VOLUME_NAME" --size "$VOLUME_SIZE" --region "$REGION" --app "$APP_NAME" --yes
    else
      echo "Volume $VOLUME_NAME already exists"
    fi

    # Set secrets (prompt user)
    echo ""
    echo "Setting up admin credentials..."
    read -p "Admin username [admin]: " ADMIN_USER
    ADMIN_USER=${ADMIN_USER:-admin}

    read -s -p "Admin password: " ADMIN_PASS
    echo ""

    if [ -z "$ADMIN_PASS" ]; then
      echo "Error: Admin password is required"
      exit 1
    fi

    flyctl secrets set \
      VAIS_REGISTRY_ADMIN_USER="$ADMIN_USER" \
      VAIS_REGISTRY_ADMIN_PASS="$ADMIN_PASS" \
      --app "$APP_NAME"

    echo ""
    echo "Setup complete! Run './scripts/fly-deploy.sh deploy' to deploy."
    ;;

  deploy)
    echo "Deploying Vais Registry to Fly.io..."
    flyctl deploy --app "$APP_NAME"

    echo ""
    echo "Deployment complete!"
    echo "Registry URL: https://$APP_NAME.fly.dev"
    echo ""
    echo "Test with:"
    echo "  curl https://$APP_NAME.fly.dev/health"
    echo "  curl https://$APP_NAME.fly.dev/api/v1/search"
    ;;

  logs)
    flyctl logs --app "$APP_NAME"
    ;;

  status)
    flyctl status --app "$APP_NAME"
    ;;

  ssh)
    flyctl ssh console --app "$APP_NAME"
    ;;

  destroy)
    echo "WARNING: This will destroy the app and all data!"
    read -p "Type 'yes' to confirm: " CONFIRM
    if [ "$CONFIRM" = "yes" ]; then
      flyctl apps destroy "$APP_NAME" --yes
    else
      echo "Aborted"
    fi
    ;;

  *)
    echo "Unknown command: $1"
    echo "Usage: $0 [setup|deploy|logs|status|ssh|destroy]"
    exit 1
    ;;
esac
