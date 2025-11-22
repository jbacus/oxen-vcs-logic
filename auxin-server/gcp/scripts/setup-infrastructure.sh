#!/bin/bash
# Set up GCP infrastructure for auxin-server using Terraform
# Usage: ./setup-infrastructure.sh [dev|staging|prod]

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

echo -e "${GREEN}=== Setting up GCP Infrastructure with Terraform ===${NC}"
echo "Environment: $ENVIRONMENT"
echo "Terraform directory: $TERRAFORM_DIR"
echo ""

# Check prerequisites
echo "Checking prerequisites..."

if ! command -v terraform &> /dev/null; then
    echo -e "${RED}Error: Terraform not found. Please install it first.${NC}"
    echo "Visit: https://www.terraform.io/downloads"
    exit 1
fi

if ! command -v gcloud &> /dev/null; then
    echo -e "${RED}Error: gcloud CLI not found. Please install it first.${NC}"
    exit 1
fi

# Check if terraform.tfvars exists
if [[ ! -f "$TERRAFORM_DIR/terraform.tfvars" ]]; then
    echo -e "${YELLOW}Warning: terraform.tfvars not found.${NC}"
    echo "Creating from example..."

    if [[ -f "$TERRAFORM_DIR/terraform.tfvars.example" ]]; then
        cp "$TERRAFORM_DIR/terraform.tfvars.example" "$TERRAFORM_DIR/terraform.tfvars"
        echo -e "${YELLOW}Please edit $TERRAFORM_DIR/terraform.tfvars with your values.${NC}"
        echo "Required fields:"
        echo "  - project_id"
        echo "  - auth_token_secret"
        echo ""
        read -p "Press Enter to open the file in your default editor..."
        ${EDITOR:-nano} "$TERRAFORM_DIR/terraform.tfvars"
    else
        echo -e "${RED}Error: terraform.tfvars.example not found${NC}"
        exit 1
    fi
fi

# Change to Terraform directory
cd "$TERRAFORM_DIR"

# Initialize Terraform
echo -e "${GREEN}Initializing Terraform...${NC}"
terraform init

# Validate configuration
echo -e "${GREEN}Validating Terraform configuration...${NC}"
terraform validate

# Show plan
echo -e "${GREEN}Generating Terraform plan...${NC}"
terraform plan -var="environment=$ENVIRONMENT"

# Confirm before applying
echo ""
read -p "Do you want to apply this plan? (yes/no): " CONFIRM

if [[ "$CONFIRM" != "yes" ]]; then
    echo "Deployment cancelled."
    exit 0
fi

# Apply configuration
echo -e "${GREEN}Applying Terraform configuration...${NC}"
terraform apply -var="environment=$ENVIRONMENT" -auto-approve

# Show outputs
echo ""
echo -e "${GREEN}=== Infrastructure Setup Complete ===${NC}"
echo ""
terraform output

echo ""
echo "Next steps:"
echo "  1. Deploy the application: $SCRIPT_DIR/deploy.sh $ENVIRONMENT"
echo "  2. View resources in GCP Console"
echo "  3. Configure CI/CD with GitHub Actions"
