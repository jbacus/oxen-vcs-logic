#!/bin/bash
# Teardown GCP infrastructure for auxin-server
# Usage: ./teardown.sh [dev|staging|prod]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TERRAFORM_DIR="$(cd "$SCRIPT_DIR/../terraform" && pwd)"
ENVIRONMENT="${1:-dev}"

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(dev|staging|prod)$ ]]; then
    echo -e "${RED}Error: Invalid environment '$ENVIRONMENT'${NC}"
    echo "Usage: $0 [dev|staging|prod]"
    exit 1
fi

echo -e "${RED}=== WARNING: Infrastructure Teardown ===${NC}"
echo "Environment: $ENVIRONMENT"
echo ""
echo "This will destroy ALL resources created by Terraform for this environment."
echo "This action is IRREVERSIBLE and will result in:"
echo "  - Cloud Run service deletion"
echo "  - Storage bucket deletion (with all data)"
echo "  - Artifact Registry deletion (with all images)"
echo "  - Secret Manager secrets deletion"
echo "  - Service accounts deletion"
echo ""

# Double confirmation for production
if [[ "$ENVIRONMENT" == "prod" ]]; then
    echo -e "${RED}⚠️  YOU ARE ABOUT TO DESTROY PRODUCTION INFRASTRUCTURE ⚠️${NC}"
    echo ""
    read -p "Type 'destroy-production' to confirm: " CONFIRM1

    if [[ "$CONFIRM1" != "destroy-production" ]]; then
        echo "Teardown cancelled."
        exit 0
    fi
fi

read -p "Type 'yes' to confirm destruction of $ENVIRONMENT infrastructure: " CONFIRM2

if [[ "$CONFIRM2" != "yes" ]]; then
    echo "Teardown cancelled."
    exit 0
fi

# Change to Terraform directory
cd "$TERRAFORM_DIR"

# Show what will be destroyed
echo -e "${GREEN}Generating destruction plan...${NC}"
terraform plan -destroy -var="environment=$ENVIRONMENT"

# Final confirmation
echo ""
read -p "Proceed with destruction? (yes/no): " FINAL_CONFIRM

if [[ "$FINAL_CONFIRM" != "yes" ]]; then
    echo "Teardown cancelled."
    exit 0
fi

# Destroy infrastructure
echo -e "${GREEN}Destroying infrastructure...${NC}"
terraform destroy -var="environment=$ENVIRONMENT" -auto-approve

echo ""
echo -e "${GREEN}Infrastructure teardown complete.${NC}"
