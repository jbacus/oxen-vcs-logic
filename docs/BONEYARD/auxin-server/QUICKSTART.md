# Auxin Server GCP Deployment - Quick Start

Get auxin-server running on Google Cloud Platform in 5 minutes.

## Prerequisites

- Google Cloud account with billing enabled
- `gcloud` CLI installed ([install guide](https://cloud.google.com/sdk/docs/install))
- Project with necessary APIs enabled

## Step 1: Initial Setup (2 minutes)

```bash
# Set your project ID
export GCP_PROJECT_ID="your-project-id"

# Set the region (optional, default: us-central1)
export GCP_REGION="us-central1"

# Authenticate
gcloud auth login
gcloud config set project $GCP_PROJECT_ID

# Enable required APIs
gcloud services enable \
  run.googleapis.com \
  artifactregistry.googleapis.com \
  cloudbuild.googleapis.com \
  secretmanager.googleapis.com \
  storage.googleapis.com
```

## Step 2: Create Infrastructure (2 minutes)

```bash
# Navigate to terraform directory
cd auxin-server/gcp/terraform

# Copy example variables
cp terraform.tfvars.example terraform.tfvars

# Edit with your values (minimal required: project_id and auth_token_secret)
nano terraform.tfvars

# Initialize and apply
terraform init
terraform apply -var="environment=dev"
```

## Step 3: Deploy the Application (1 minute)

```bash
# Navigate to scripts directory
cd ../scripts

# Deploy to Cloud Run
./deploy.sh dev
```

## Step 4: Verify Deployment

```bash
# Get the service URL (from deploy.sh output)
export SERVICE_URL=$(gcloud run services describe auxin-server \
  --region $GCP_REGION \
  --format 'value(status.url)')

# Test health endpoint
curl $SERVICE_URL/health

# Expected output:
# {"status":"healthy"}
```

## Next Steps

1. **Configure DNS** (optional)
   ```bash
   gcloud run domain-mappings create \
     --service auxin-server \
     --domain your-domain.com \
     --region $GCP_REGION
   ```

2. **Set up CI/CD** - See [DEPLOYMENT.md](DEPLOYMENT.md#cicd-with-github-actions)

3. **Monitor your service**
   ```bash
   # View logs
   gcloud run services logs tail auxin-server --region $GCP_REGION

   # Open in browser
   open "https://console.cloud.google.com/run/detail/$GCP_REGION/auxin-server"
   ```

4. **Scale as needed** - Edit `terraform.tfvars` and reapply

## Minimal terraform.tfvars

```hcl
project_id = "your-gcp-project-id"
region     = "us-central1"
environment = "dev"

# Generate a secure secret:
# openssl rand -base64 32
auth_token_secret = "your-super-secret-token-here"

# Optional: Allow public access for testing
allow_public_access = true
```

## Troubleshooting

### Issue: Permission denied

```bash
# Grant yourself necessary roles
gcloud projects add-iam-policy-binding $GCP_PROJECT_ID \
  --member="user:your-email@example.com" \
  --role="roles/run.admin"
```

### Issue: API not enabled

```bash
# Check enabled APIs
gcloud services list --enabled

# Enable missing APIs
gcloud services enable <api-name>
```

### Issue: Build fails

```bash
# Check build logs
gcloud builds list --limit 5
gcloud builds log <BUILD_ID>
```

## Clean Up

To avoid charges, destroy resources when done testing:

```bash
cd auxin-server/gcp/scripts
./teardown.sh dev
```

## Cost Estimate

Development environment with minimal usage: **~$6-15/month**

Free tier includes:
- 2 million Cloud Run requests/month
- 400,000 GB-seconds/month
- 5 GB Cloud Storage

## More Information

- [Full Deployment Guide](DEPLOYMENT.md)
- [Architecture Overview](README.md#architecture)
- [Monitoring Guide](README.md#monitoring)
