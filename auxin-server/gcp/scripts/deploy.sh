#!/bin/bash
# Deploy auxin-server to Google Cloud Platform
# Usage: ./deploy.sh [dev|staging|prod]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
SERVER_DIR="$PROJECT_ROOT/auxin-server"
TERRAFORM_DIR="$SERVER_DIR/gcp/terraform"

# Default values
ENVIRONMENT="${1:-dev}"
GCP_REGION="${GCP_REGION:-us-central1}"
ARTIFACT_REGISTRY_REPO="auxin"
SERVICE_NAME="auxin-server"

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(dev|staging|prod)$ ]]; then
    echo -e "${RED}Error: Invalid environment '$ENVIRONMENT'${NC}"
    echo "Usage: $0 [dev|staging|prod]"
    exit 1
fi

# Load environment-specific configuration
if [[ -f "$TERRAFORM_DIR/terraform.tfvars" ]]; then
    # Extract project ID from terraform.tfvars
    GCP_PROJECT_ID=$(grep -E '^project_id' "$TERRAFORM_DIR/terraform.tfvars" | cut -d'"' -f2)
else
    echo -e "${YELLOW}Warning: terraform.tfvars not found. Using gcloud default project.${NC}"
    GCP_PROJECT_ID=$(gcloud config get-value project)
fi

if [[ -z "$GCP_PROJECT_ID" ]]; then
    echo -e "${RED}Error: GCP_PROJECT_ID not set. Please configure gcloud or create terraform.tfvars${NC}"
    exit 1
fi

echo -e "${GREEN}=== Deploying Auxin Server to GCP ===${NC}"
echo "Environment: $ENVIRONMENT"
echo "Project: $GCP_PROJECT_ID"
echo "Region: $GCP_REGION"
echo ""

# Check prerequisites
echo "Checking prerequisites..."

if ! command -v gcloud &> /dev/null; then
    echo -e "${RED}Error: gcloud CLI not found. Please install it first.${NC}"
    exit 1
fi

if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker not found. Please install it first.${NC}"
    exit 1
fi

# Authenticate with gcloud
echo "Authenticating with Google Cloud..."
gcloud config set project "$GCP_PROJECT_ID"

# Configure Docker for Artifact Registry
echo "Configuring Docker for Artifact Registry..."
gcloud auth configure-docker "${GCP_REGION}-docker.pkg.dev" --quiet

# Build Docker image
echo -e "${GREEN}Building Docker image...${NC}"
IMAGE_TAG="${GCP_REGION}-docker.pkg.dev/${GCP_PROJECT_ID}/${ARTIFACT_REGISTRY_REPO}/${SERVICE_NAME}"
COMMIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "local")

cd "$SERVER_DIR"
docker build \
    -t "${IMAGE_TAG}:${COMMIT_SHA}" \
    -t "${IMAGE_TAG}:latest" \
    -f Dockerfile \
    .

# Push Docker image
echo -e "${GREEN}Pushing Docker image to Artifact Registry...${NC}"
docker push --all-tags "$IMAGE_TAG"

# Environment-specific deployment configuration
case "$ENVIRONMENT" in
    dev)
        MEMORY="2Gi"
        CPU="2"
        MIN_INSTANCES="0"
        MAX_INSTANCES="10"
        ALLOW_UNAUTH="--allow-unauthenticated"
        ;;
    staging)
        MEMORY="4Gi"
        CPU="2"
        MIN_INSTANCES="1"
        MAX_INSTANCES="20"
        ALLOW_UNAUTH="--no-allow-unauthenticated"
        SERVICE_NAME="${SERVICE_NAME}-staging"
        ;;
    prod)
        MEMORY="8Gi"
        CPU="4"
        MIN_INSTANCES="2"
        MAX_INSTANCES="50"
        ALLOW_UNAUTH="--no-allow-unauthenticated"
        ;;
esac

# Deploy to Cloud Run
echo -e "${GREEN}Deploying to Cloud Run (${ENVIRONMENT})...${NC}"
gcloud run deploy "$SERVICE_NAME" \
    --image "${IMAGE_TAG}:${COMMIT_SHA}" \
    --region "$GCP_REGION" \
    --platform managed \
    $ALLOW_UNAUTH \
    --memory "$MEMORY" \
    --cpu "$CPU" \
    --min-instances "$MIN_INSTANCES" \
    --max-instances "$MAX_INSTANCES" \
    --port 3000 \
    --timeout 300 \
    --set-env-vars "RUST_LOG=info,auxin_server=debug,OXEN_SERVER_PORT=3000,OXEN_SERVER_HOST=0.0.0.0" \
    --set-secrets "AUTH_TOKEN_SECRET=auxin-auth-secret:latest"

# Get service URL
SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" \
    --region "$GCP_REGION" \
    --format 'value(status.url)')

echo ""
echo -e "${GREEN}=== Deployment Complete ===${NC}"
echo "Service URL: $SERVICE_URL"
echo ""
echo "Testing health endpoint..."
sleep 5
if curl -f "$SERVICE_URL/health" &> /dev/null; then
    echo -e "${GREEN}✓ Health check passed!${NC}"
else
    echo -e "${YELLOW}⚠ Health check failed. Service may still be starting up.${NC}"
fi

echo ""
echo "Next steps:"
echo "  - View logs: gcloud run services logs read $SERVICE_NAME --region $GCP_REGION"
echo "  - Open in browser: $SERVICE_URL"
echo "  - View in console: https://console.cloud.google.com/run/detail/$GCP_REGION/$SERVICE_NAME"
