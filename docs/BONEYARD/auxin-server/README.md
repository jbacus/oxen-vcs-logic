# Auxin Server - Google Cloud Platform Deployment

This directory contains all the necessary configuration and scripts to deploy auxin-server to Google Cloud Platform (GCP).

## 📁 Directory Structure

```
gcp/
├── README.md                    # This file
├── DEPLOYMENT.md                # Comprehensive deployment guide
├── QUICKSTART.md                # Quick start guide
├── cloudbuild.yaml              # Cloud Build configuration
├── terraform/                   # Infrastructure as Code
│   ├── main.tf                  # Main Terraform configuration
│   ├── variables.tf             # Variable definitions
│   ├── outputs.tf               # Output values
│   ├── terraform.tfvars.example # Example variables file
│   └── .gitignore               # Terraform gitignore
└── scripts/                     # Deployment scripts
    ├── deploy.sh                # Deploy to Cloud Run
    ├── setup-infrastructure.sh  # Set up infrastructure with Terraform
    └── teardown.sh              # Destroy infrastructure

```

## 🚀 Quick Start

### Prerequisites

1. **Google Cloud Project** with billing enabled
2. **gcloud CLI** installed and authenticated
3. **Terraform** >= 1.5 (for infrastructure setup)
4. **Docker** (for local builds)

### Option 1: One-Command Deploy

```bash
# Set up infrastructure and deploy
cd gcp/scripts
./setup-infrastructure.sh dev
./deploy.sh dev
```

### Option 2: Use GitHub Actions

1. Configure secrets in GitHub (see DEPLOYMENT.md)
2. Push to `main` branch
3. Automatic deployment to dev environment

### Option 3: Manual Cloud Build

```bash
gcloud builds submit --config=gcp/cloudbuild.yaml auxin-server/
```

## 📚 Documentation

- **[QUICKSTART.md](QUICKSTART.md)** - Get started in 5 minutes
- **[DEPLOYMENT.md](DEPLOYMENT.md)** - Comprehensive deployment guide
- **[../DEPLOYMENT.md](../DEPLOYMENT.md)** - General deployment guide (Docker, local)

## 🏗️ Architecture

### Components

1. **Cloud Run** - Serverless container hosting
   - Automatic scaling (0-50 instances)
   - Pay-per-use pricing
   - HTTPS endpoints with SSL

2. **Artifact Registry** - Docker image storage
   - Multi-region replication
   - Vulnerability scanning
   - Access control

3. **Cloud Storage** - Oxen data persistence
   - 90-day lifecycle management
   - Versioning enabled
   - Regional storage

4. **Secret Manager** - Secure configuration
   - Encrypted secrets
   - Automatic rotation support
   - IAM-based access control

5. **VPC Connector** (optional) - Private networking
   - Access to private resources
   - VPN connectivity

### Environments

| Environment | Branch | Instances | Memory | CPU | Public Access |
|-------------|--------|-----------|--------|-----|---------------|
| **dev** | main | 0-10 | 2Gi | 2 | ✓ |
| **staging** | release/* | 1-20 | 4Gi | 2 | ✗ |
| **prod** | manual | 2-50 | 8Gi | 4 | ✗ |

## 💰 Cost Estimation

### Development (Low Usage)
- Cloud Run: ~$5-10/month
- Cloud Storage: ~$1-5/month
- Artifact Registry: ~$0.10/GB/month
- **Total: ~$6-15/month**

### Production (Moderate Usage)
- Cloud Run: ~$50-100/month
- Cloud Storage: ~$20-50/month
- Artifact Registry: ~$5-10/month
- **Total: ~$75-160/month**

*Costs vary based on usage, region, and configuration.*

## 🔒 Security

### Best Practices Implemented

- ✅ Non-root container user
- ✅ Secrets in Secret Manager (not environment variables)
- ✅ IAM-based access control
- ✅ HTTPS-only traffic
- ✅ No hardcoded credentials
- ✅ Vulnerability scanning on images
- ✅ Least-privilege service accounts

### Authentication Options

1. **Public Access** (dev only)
   - `--allow-unauthenticated` flag
   - Good for testing

2. **IAM-based** (staging/prod)
   - Requires authentication
   - User/service account authorization

3. **Cloud Armor** (optional)
   - DDoS protection
   - WAF rules
   - Rate limiting

## 🔧 Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SYNC_DIR` | Oxen data directory | `/var/oxen/data` |
| `OXEN_SERVER_PORT` | Server port | `3000` |
| `OXEN_SERVER_HOST` | Bind address | `0.0.0.0` |
| `RUST_LOG` | Log level | `info,auxin_server=debug` |
| `AUTH_TOKEN_SECRET` | JWT secret (from Secret Manager) | - |

### Terraform Variables

See `terraform/terraform.tfvars.example` for all configuration options.

Key variables:
- `project_id` - Your GCP project ID
- `region` - GCP region (default: us-central1)
- `environment` - dev/staging/prod
- `allow_public_access` - Enable public access
- `enable_auto_deploy` - Enable GitHub Actions CI/CD

## 🧪 Testing

### Smoke Tests

```bash
# Check health endpoint
curl https://your-service-url.run.app/health

# Test API
curl https://your-service-url.run.app/api/repos
```

### Load Testing

```bash
# Using Apache Bench
ab -n 1000 -c 10 https://your-service-url.run.app/health

# Using wrk
wrk -t12 -c400 -d30s https://your-service-url.run.app/health
```

## 📊 Monitoring

### View Logs

```bash
# Stream logs
gcloud run services logs tail auxin-server --region us-central1

# View recent logs
gcloud run services logs read auxin-server --region us-central1 --limit 50
```

### Metrics

Access in Cloud Console:
- https://console.cloud.google.com/run/detail/us-central1/auxin-server/metrics

Key metrics:
- Request count
- Request latency
- Instance count
- CPU utilization
- Memory utilization

### Alerts (Optional)

Set up alerts for:
- High error rate (> 5%)
- High latency (> 2s p99)
- Low availability (< 99%)

## 🛠️ Troubleshooting

### Common Issues

1. **Build fails**
   ```bash
   # Check build logs
   gcloud builds list --limit 10
   gcloud builds log <BUILD_ID>
   ```

2. **Deployment fails**
   ```bash
   # Check service status
   gcloud run services describe auxin-server --region us-central1

   # View events
   gcloud run services describe auxin-server --region us-central1 --format yaml
   ```

3. **Service errors**
   ```bash
   # Stream logs for errors
   gcloud run services logs tail auxin-server --region us-central1 | grep ERROR
   ```

### Debug Mode

Enable verbose logging:
```bash
gcloud run services update auxin-server \
  --set-env-vars RUST_LOG=debug,auxin_server=trace \
  --region us-central1
```

## 📞 Support

- **Issues**: https://github.com/jbacus/auxin/issues
- **Discussions**: https://github.com/jbacus/auxin/discussions
- **Documentation**: ../DEPLOYMENT.md

## 📄 License

See [LICENSE](../../LICENSE) in the root directory.
