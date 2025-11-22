# Auxin Server - Comprehensive GCP Deployment Guide

This guide covers everything you need to deploy, configure, and manage auxin-server on Google Cloud Platform.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Architecture Overview](#architecture-overview)
3. [Initial Setup](#initial-setup)
4. [Infrastructure Deployment](#infrastructure-deployment)
5. [Application Deployment](#application-deployment)
6. [CI/CD with GitHub Actions](#cicd-with-github-actions)
7. [Configuration](#configuration)
8. [Monitoring & Logging](#monitoring--logging)
9. [Security](#security)
10. [Scaling](#scaling)
11. [Troubleshooting](#troubleshooting)
12. [Cost Optimization](#cost-optimization)

---

## Prerequisites

### Required Tools

1. **Google Cloud SDK (`gcloud`)**
   ```bash
   # Install on macOS
   brew install google-cloud-sdk

   # Install on Linux
   curl https://sdk.cloud.google.com | bash

   # Verify installation
   gcloud version
   ```

2. **Terraform** >= 1.5
   ```bash
   # Install on macOS
   brew install terraform

   # Install on Linux
   wget https://releases.hashicorp.com/terraform/1.6.0/terraform_1.6.0_linux_amd64.zip
   unzip terraform_1.6.0_linux_amd64.zip
   sudo mv terraform /usr/local/bin/

   # Verify installation
   terraform version
   ```

3. **Docker**
   ```bash
   # Install on macOS
   brew install docker

   # Verify installation
   docker --version
   ```

### Google Cloud Setup

1. **Create GCP Project**
   ```bash
   # Create new project
   gcloud projects create auxin-server-dev --name="Auxin Server Dev"

   # Set as default
   gcloud config set project auxin-server-dev

   # Link billing account (required)
   gcloud billing accounts list
   gcloud billing projects link auxin-server-dev \
     --billing-account=YOUR_BILLING_ACCOUNT_ID
   ```

2. **Authenticate**
   ```bash
   # Login
   gcloud auth login

   # Set up application default credentials
   gcloud auth application-default login
   ```

3. **Enable Required APIs**
   ```bash
   gcloud services enable \
     run.googleapis.com \
     artifactregistry.googleapis.com \
     cloudbuild.googleapis.com \
     secretmanager.googleapis.com \
     storage.googleapis.com \
     vpcaccess.googleapis.com \
     compute.googleapis.com
   ```

---

## Architecture Overview

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Google Cloud Platform                    │
│                                                              │
│  ┌──────────────┐      ┌──────────────┐                    │
│  │   GitHub     │─────▶│ Cloud Build  │                    │
│  │   Actions    │      │   (CI/CD)    │                    │
│  └──────────────┘      └──────┬───────┘                    │
│                                │                             │
│                                ▼                             │
│                       ┌────────────────┐                    │
│                       │   Artifact     │                    │
│                       │   Registry     │                    │
│                       └───────┬────────┘                    │
│                               │                              │
│                               ▼                              │
│  ┌────────────────────────────────────────────┐            │
│  │           Cloud Run Service                │            │
│  │  ┌──────────────────────────────────────┐ │            │
│  │  │      Auxin Server Container          │ │            │
│  │  │  • Rust Backend (Actix-Web)          │ │            │
│  │  │  • React Frontend                    │ │            │
│  │  │  • Auto-scaling (0-50 instances)     │ │            │
│  │  └──────────────────────────────────────┘ │            │
│  └────────┬───────────────┬───────────────────┘            │
│           │               │                                 │
│           ▼               ▼                                 │
│  ┌────────────┐   ┌──────────────┐                        │
│  │   Cloud    │   │    Secret    │                        │
│  │  Storage   │   │   Manager    │                        │
│  │  (Oxen     │   │  (Auth       │                        │
│  │   Data)    │   │  Secrets)    │                        │
│  └────────────┘   └──────────────┘                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Service Architecture

- **Cloud Run**: Fully managed, serverless container platform
- **Artifact Registry**: Private Docker registry for container images
- **Cloud Storage**: Object storage for Oxen repository data
- **Secret Manager**: Encrypted secrets management
- **VPC Connector**: Optional private networking (for databases, Redis, etc.)

---

## Initial Setup

### 1. Configure Terraform Variables

```bash
cd auxin-server/gcp/terraform
cp terraform.tfvars.example terraform.tfvars
```

Edit `terraform.tfvars`:

```hcl
# Required
project_id = "your-gcp-project-id"
region     = "us-central1"
environment = "dev"

# Generate with: openssl rand -base64 32
auth_token_secret = "your-secure-secret-here"

# Optional
allow_public_access = true  # false for staging/prod
enable_versioning   = true
enable_vpc_connector = false
enable_auto_deploy  = true

# For CI/CD
github_owner = "your-github-username"
github_repo  = "auxin"

# Scaling
min_instances = 0  # 0 for dev, 1+ for staging/prod
max_instances = 10 # Adjust based on expected load
cpu           = "2"
memory        = "2Gi"
```

### 2. Initialize Terraform Backend (Optional)

For team environments, use remote state:

```bash
# Create state bucket
gsutil mb gs://auxin-terraform-state

# Enable versioning
gsutil versioning set on gs://auxin-terraform-state

# Update main.tf
# Uncomment and configure the backend "gcs" block
```

---

## Infrastructure Deployment

### Using Terraform Directly

```bash
cd auxin-server/gcp/terraform

# Initialize
terraform init

# Plan
terraform plan -var="environment=dev"

# Apply
terraform apply -var="environment=dev"

# Save outputs
terraform output > outputs.txt
```

### Using Setup Script

```bash
cd auxin-server/gcp/scripts
./setup-infrastructure.sh dev
```

### Verify Infrastructure

```bash
# Check Cloud Run service
gcloud run services list --region us-central1

# Check Artifact Registry
gcloud artifacts repositories list --location us-central1

# Check Cloud Storage buckets
gsutil ls

# Check secrets
gcloud secrets list
```

---

## Application Deployment

### Method 1: Deployment Script (Recommended)

```bash
cd auxin-server/gcp/scripts
./deploy.sh dev
```

This script:
1. Builds Docker image
2. Pushes to Artifact Registry
3. Deploys to Cloud Run
4. Runs health checks

### Method 2: Manual Deployment

```bash
# Set variables
PROJECT_ID="your-project-id"
REGION="us-central1"
SERVICE_NAME="auxin-server"

# Build image
cd auxin-server
docker build -t ${REGION}-docker.pkg.dev/${PROJECT_ID}/auxin/${SERVICE_NAME}:latest .

# Configure Docker auth
gcloud auth configure-docker ${REGION}-docker.pkg.dev

# Push image
docker push ${REGION}-docker.pkg.dev/${PROJECT_ID}/auxin/${SERVICE_NAME}:latest

# Deploy to Cloud Run
gcloud run deploy ${SERVICE_NAME} \
  --image ${REGION}-docker.pkg.dev/${PROJECT_ID}/auxin/${SERVICE_NAME}:latest \
  --region ${REGION} \
  --platform managed \
  --allow-unauthenticated \
  --memory 2Gi \
  --cpu 2 \
  --min-instances 0 \
  --max-instances 10 \
  --port 3000 \
  --timeout 300 \
  --set-env-vars "RUST_LOG=info,auxin_server=debug" \
  --set-secrets "AUTH_TOKEN_SECRET=auxin-auth-secret:latest"
```

### Method 3: Cloud Build

```bash
# Submit build
gcloud builds submit \
  --config=auxin-server/gcp/cloudbuild.yaml \
  auxin-server/

# With substitutions
gcloud builds submit \
  --config=auxin-server/gcp/cloudbuild.yaml \
  --substitutions=_REGION=us-central1,_SERVICE_NAME=auxin-server \
  auxin-server/
```

---

## CI/CD with GitHub Actions

### 1. Set Up Workload Identity Federation (Recommended)

Workload Identity Federation allows GitHub Actions to authenticate without service account keys.

```bash
# Set variables
PROJECT_ID="your-project-id"
PROJECT_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")
POOL_NAME="github-pool"
PROVIDER_NAME="github-provider"
SERVICE_ACCOUNT_NAME="github-actions"
REPO="your-github-username/auxin"

# Create Workload Identity Pool
gcloud iam workload-identity-pools create $POOL_NAME \
  --location="global" \
  --display-name="GitHub Actions Pool"

# Create Workload Identity Provider
gcloud iam workload-identity-pools providers create-oidc $PROVIDER_NAME \
  --location="global" \
  --workload-identity-pool=$POOL_NAME \
  --display-name="GitHub Provider" \
  --attribute-mapping="google.subject=assertion.sub,attribute.actor=assertion.actor,attribute.repository=assertion.repository" \
  --issuer-uri="https://token.actions.githubusercontent.com"

# Create service account
gcloud iam service-accounts create $SERVICE_ACCOUNT_NAME \
  --display-name="GitHub Actions Service Account"

# Grant permissions
gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:${SERVICE_ACCOUNT_NAME}@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/run.admin"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:${SERVICE_ACCOUNT_NAME}@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/iam.serviceAccountUser"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:${SERVICE_ACCOUNT_NAME}@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/artifactregistry.admin"

# Allow GitHub to impersonate service account
gcloud iam service-accounts add-iam-policy-binding \
  "${SERVICE_ACCOUNT_NAME}@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/iam.workloadIdentityUser" \
  --member="principalSet://iam.googleapis.com/projects/${PROJECT_NUMBER}/locations/global/workloadIdentityPools/${POOL_NAME}/attribute.repository/${REPO}"

# Get Workload Identity Provider resource name
gcloud iam workload-identity-pools providers describe $PROVIDER_NAME \
  --location="global" \
  --workload-identity-pool=$POOL_NAME \
  --format="value(name)"
```

### 2. Configure GitHub Secrets

Add these secrets to your GitHub repository (Settings > Secrets and variables > Actions):

**For Dev/Staging:**
- `GCP_PROJECT_ID`: Your GCP project ID
- `GCP_WORKLOAD_IDENTITY_PROVIDER`: Workload Identity Provider resource name
- `GCP_SERVICE_ACCOUNT`: Service account email

**For Production (separate project):**
- `GCP_PROJECT_ID_PROD`
- `GCP_WORKLOAD_IDENTITY_PROVIDER_PROD`
- `GCP_SERVICE_ACCOUNT_PROD`

### 3. Trigger Deployment

```bash
# Automatic on push to main
git push origin main

# Manual dispatch
gh workflow run deploy-auxin-server-gcp.yml

# With environment selection
gh workflow run deploy-auxin-server-gcp.yml -f environment=staging
```

---

## Configuration

### Environment Variables

Set via Cloud Run:

```bash
gcloud run services update auxin-server \
  --region us-central1 \
  --set-env-vars "KEY1=value1,KEY2=value2"
```

Available variables:
- `SYNC_DIR`: Oxen data directory (default: `/var/oxen/data`)
- `OXEN_SERVER_PORT`: Server port (default: `3000`)
- `OXEN_SERVER_HOST`: Bind address (default: `0.0.0.0`)
- `RUST_LOG`: Log level (default: `info,auxin_server=debug`)

### Secrets

Manage via Secret Manager:

```bash
# Create secret
echo -n "my-secret-value" | gcloud secrets create auxin-auth-secret --data-file=-

# Update secret
echo -n "new-secret-value" | gcloud secrets versions add auxin-auth-secret --data-file=-

# Grant access
gcloud secrets add-iam-policy-binding auxin-auth-secret \
  --member="serviceAccount:auxin-server@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"

# Use in Cloud Run
gcloud run services update auxin-server \
  --region us-central1 \
  --set-secrets "AUTH_TOKEN_SECRET=auxin-auth-secret:latest"
```

---

## Monitoring & Logging

### View Logs

```bash
# Stream logs
gcloud run services logs tail auxin-server --region us-central1

# View recent logs
gcloud run services logs read auxin-server --region us-central1 --limit 100

# Filter by severity
gcloud run services logs read auxin-server --region us-central1 --log-filter="severity>=ERROR"

# Export to file
gcloud run services logs read auxin-server --region us-central1 > logs.txt
```

### Metrics Dashboard

Access in Cloud Console:
```
https://console.cloud.google.com/run/detail/us-central1/auxin-server/metrics
```

Key metrics:
- **Request count**: Total requests per minute
- **Request latency**: P50, P95, P99 latency
- **Error rate**: 4xx and 5xx responses
- **Instance count**: Active container instances
- **CPU utilization**: CPU usage percentage
- **Memory utilization**: Memory usage

### Set Up Alerts

```bash
# Create alert policy for high error rate
gcloud alpha monitoring policies create \
  --notification-channels=CHANNEL_ID \
  --display-name="Auxin Server High Error Rate" \
  --condition-display-name="Error rate > 5%" \
  --condition-threshold-value=0.05 \
  --condition-threshold-duration=300s
```

### Structured Logging

Auxin server uses structured JSON logging. Example query in Logs Explorer:

```
resource.type="cloud_run_revision"
resource.labels.service_name="auxin-server"
severity="ERROR"
```

---

## Security

### Authentication Options

#### 1. Public Access (Dev Only)

```bash
gcloud run services add-iam-policy-binding auxin-server \
  --region us-central1 \
  --member="allUsers" \
  --role="roles/run.invoker"
```

#### 2. Authenticated Users Only

```bash
# Remove public access
gcloud run services remove-iam-policy-binding auxin-server \
  --region us-central1 \
  --member="allUsers" \
  --role="roles/run.invoker"

# Add specific user
gcloud run services add-iam-policy-binding auxin-server \
  --region us-central1 \
  --member="user:alice@example.com" \
  --role="roles/run.invoker"

# Call with authentication
curl -H "Authorization: Bearer $(gcloud auth print-identity-token)" \
  https://auxin-server-xxx.run.app/health
```

#### 3. Service Account Only

```bash
# Create invoker service account
gcloud iam service-accounts create auxin-invoker

# Grant access
gcloud run services add-iam-policy-binding auxin-server \
  --region us-central1 \
  --member="serviceAccount:auxin-invoker@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/run.invoker"
```

### Network Security

#### Enable VPC Connector

```hcl
# In terraform.tfvars
enable_vpc_connector = true
```

This allows:
- Access to private Cloud SQL databases
- Access to Redis/Memorystore
- VPN connectivity

#### Cloud Armor (WAF)

```bash
# Create security policy
gcloud compute security-policies create auxin-waf

# Add rate limiting rule
gcloud compute security-policies rules create 1000 \
  --security-policy auxin-waf \
  --expression "true" \
  --action "rate-based-ban" \
  --rate-limit-threshold-count 100 \
  --rate-limit-threshold-interval-sec 60 \
  --ban-duration-sec 600

# Apply to load balancer (requires NEG setup)
```

### Vulnerability Scanning

Artifact Registry automatically scans images:

```bash
# View scan results
gcloud artifacts docker images list us-central1-docker.pkg.dev/${PROJECT_ID}/auxin

# View vulnerabilities
gcloud artifacts docker images describe \
  us-central1-docker.pkg.dev/${PROJECT_ID}/auxin/auxin-server:latest \
  --show-all-metadata
```

---

## Scaling

### Horizontal Scaling (Instances)

```bash
# Update instance limits
gcloud run services update auxin-server \
  --region us-central1 \
  --min-instances 2 \
  --max-instances 50
```

Configuration by environment:

| Environment | Min | Max | Reason |
|-------------|-----|-----|--------|
| Dev | 0 | 10 | Cost optimization |
| Staging | 1 | 20 | Always available, moderate load |
| Prod | 2 | 50 | High availability, handle traffic spikes |

### Vertical Scaling (Resources)

```bash
# Update CPU and memory
gcloud run services update auxin-server \
  --region us-central1 \
  --cpu 4 \
  --memory 8Gi
```

Resource sizing guide:

| Load | CPU | Memory | Concurrent Requests |
|------|-----|--------|---------------------|
| Light | 1 | 512Mi | 80 |
| Medium | 2 | 2Gi | 80 |
| Heavy | 4 | 4Gi | 100 |
| Very Heavy | 8 | 8Gi | 100 |

### Concurrency

```bash
# Set max concurrent requests per instance
gcloud run services update auxin-server \
  --region us-central1 \
  --concurrency 80
```

### Cold Start Optimization

```bash
# Keep at least 1 instance warm
gcloud run services update auxin-server \
  --region us-central1 \
  --min-instances 1

# Use startup CPU boost (beta)
gcloud beta run services update auxin-server \
  --region us-central1 \
  --cpu-boost
```

---

## Troubleshooting

### Build Issues

```bash
# List recent builds
gcloud builds list --limit 10

# View build details
gcloud builds describe BUILD_ID

# View build logs
gcloud builds log BUILD_ID

# Trigger manual build
cd auxin-server
gcloud builds submit --tag us-central1-docker.pkg.dev/${PROJECT_ID}/auxin/auxin-server:debug
```

### Deployment Issues

```bash
# Check service status
gcloud run services describe auxin-server --region us-central1

# View deployment events
gcloud run revisions list \
  --service auxin-server \
  --region us-central1

# View specific revision
gcloud run revisions describe REVISION_NAME --region us-central1

# Rollback to previous revision
gcloud run services update-traffic auxin-server \
  --region us-central1 \
  --to-revisions PREVIOUS_REVISION=100
```

### Runtime Issues

```bash
# Stream logs with errors only
gcloud run services logs tail auxin-server \
  --region us-central1 \
  --log-filter "severity>=ERROR"

# Check health endpoint
curl https://your-service-url.run.app/health

# Test with verbose output
curl -v https://your-service-url.run.app/api/repos

# Check environment variables
gcloud run services describe auxin-server \
  --region us-central1 \
  --format yaml | grep -A 20 "env:"
```

### Performance Issues

```bash
# View metrics
gcloud monitoring time-series list \
  --filter 'resource.type="cloud_run_revision"' \
  --filter 'resource.labels.service_name="auxin-server"'

# Check instance count
gcloud run services describe auxin-server \
  --region us-central1 \
  --format "value(status.traffic[0].latestRevision.containerConcurrency)"

# Enable request logging
gcloud run services update auxin-server \
  --region us-central1 \
  --set-env-vars "RUST_LOG=trace"
```

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Container failed to start" | Port mismatch | Check `EXPOSE 3000` in Dockerfile |
| "Insufficient Permission" | IAM roles missing | Grant `roles/run.admin` |
| "Quota exceeded" | Resource limits | Request quota increase |
| "503 Service Unavailable" | Cold start timeout | Increase `--timeout` or use `--min-instances=1` |

---

## Cost Optimization

### Free Tier

Cloud Run free tier (per month):
- 2 million requests
- 400,000 GB-seconds (compute)
- 200,000 vCPU-seconds
- 360,000 GB-seconds (memory)

### Cost Reduction Strategies

1. **Scale to Zero**
   ```bash
   gcloud run services update auxin-server \
     --region us-central1 \
     --min-instances 0
   ```

2. **Reduce Memory/CPU for Dev**
   ```bash
   gcloud run services update auxin-server \
     --region us-central1 \
     --cpu 1 \
     --memory 512Mi
   ```

3. **Set Request Timeout**
   ```bash
   gcloud run services update auxin-server \
     --region us-central1 \
     --timeout 60  # seconds
   ```

4. **Use Regional Storage**
   ```hcl
   # In terraform.tfvars
   # Regional is cheaper than multi-regional
   ```

5. **Enable Storage Lifecycle**
   ```bash
   # Configured in Terraform: 90-day deletion
   ```

### Cost Monitoring

```bash
# View current month costs
gcloud billing accounts describe BILLING_ACCOUNT_ID

# Set up budget alerts
gcloud billing budgets create \
  --billing-account=BILLING_ACCOUNT_ID \
  --display-name="Auxin Server Budget" \
  --budget-amount=100USD
```

### Estimated Monthly Costs

**Development (minimal usage)**:
- Cloud Run: ~$5-10
- Cloud Storage: ~$1-5
- Artifact Registry: ~$0.10/GB
- **Total: ~$6-15**

**Production (moderate usage, 10K req/day)**:
- Cloud Run: ~$50-100
- Cloud Storage: ~$20-50
- Artifact Registry: ~$5-10
- **Total: ~$75-160**

---

## Additional Resources

- [Cloud Run Documentation](https://cloud.google.com/run/docs)
- [Terraform GCP Provider](https://registry.terraform.io/providers/hashicorp/google/latest/docs)
- [GitHub Actions for GCP](https://github.com/google-github-actions)
- [GCP Pricing Calculator](https://cloud.google.com/products/calculator)

---

## Support

- **Issues**: https://github.com/jbacus/auxin/issues
- **Discussions**: https://github.com/jbacus/auxin/discussions

---

*Last Updated: 2025-11-22*
