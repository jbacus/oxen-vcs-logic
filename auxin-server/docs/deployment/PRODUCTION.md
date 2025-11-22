# Auxin Server - Production Deployment Guide

**Last Updated**: 2025-11-22

This guide covers deploying Auxin Server in a production environment with Docker, Nginx reverse proxy, SSL/TLS, monitoring, and backup strategies.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Docker Deployment](#docker-deployment)
4. [Nginx Reverse Proxy](#nginx-reverse-proxy)
5. [SSL/TLS with Let's Encrypt](#ssltls-with-lets-encrypt)
6. [Systemd Service](#systemd-service)
7. [Backup and Restore](#backup-and-restore)
8. [Monitoring](#monitoring)
9. [Security Hardening](#security-hardening)
10. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### System Requirements

- **OS**: Ubuntu 22.04 LTS or later (recommended), Debian 11+, or any Linux with Docker support
- **CPU**: 2+ cores recommended
- **RAM**: 4GB minimum, 8GB+ recommended for production
- **Disk**: 50GB+ SSD for repository data
- **Network**: Static IP or domain name for SSL

### Software Requirements

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo apt install docker-compose-plugin

# Install Nginx (for reverse proxy)
sudo apt install nginx

# Install Certbot (for SSL)
sudo apt install certbot python3-certbot-nginx
```

### Domain Setup

- Register a domain (e.g., `auxin.yourdomain.com`)
- Point DNS A record to your server's IP address
- Wait for DNS propagation (15 minutes - 48 hours)

---

## Quick Start

### 1. Clone Repository

```bash
cd /opt
sudo git clone https://github.com/jbacus/auxin.git
cd auxin/auxin-server
```

### 2. Configure

```bash
# Copy example configuration
cp config.docker.toml config.toml

# Edit configuration
nano config.toml
```

**Required changes**:
```toml
[server]
auth_token_secret = "CHANGE_THIS_TO_RANDOM_STRING"  # Generate with: openssl rand -base64 32
```

### 3. Build and Run

```bash
# Build Docker image
docker compose build

# Start services
docker compose up -d

# Check logs
docker compose logs -f auxin-server
```

### 4. Verify

```bash
curl http://localhost:3000/health
# Should return: OK
```

---

## Docker Deployment

### Production docker-compose.yml

Create `/opt/auxin/auxin-server/docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  auxin-server:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: auxin-server
    restart: unless-stopped
    ports:
      - "127.0.0.1:3000:3000"  # Only expose to localhost (Nginx will proxy)
    volumes:
      - auxin-data:/var/oxen/data
      - ./config.toml:/app/config.toml:ro
      - ./logs:/var/log/auxin
    environment:
      - RUST_LOG=info,auxin_server=info
      - AUXIN_SERVER_AUTH_TOKEN_SECRET=${AUTH_SECRET}
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "5"

  redis:
    image: redis:7-alpine
    container_name: auxin-redis
    restart: unless-stopped
    ports:
      - "127.0.0.1:6379:6379"
    volumes:
      - redis-data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 30s
      timeout: 5s
      retries: 3

volumes:
  auxin-data:
    driver: local
  redis-data:
    driver: local
```

### Environment Variables

Create `.env` file (secrets only):

```bash
# NEVER commit this file to version control!
AUTH_SECRET=$(openssl rand -base64 32)
```

### Start Production Stack

```bash
docker compose -f docker-compose.prod.yml up -d

# View logs
docker compose -f docker-compose.prod.yml logs -f

# Stop
docker compose -f docker-compose.prod.yml down

# Restart
docker compose -f docker-compose.prod.yml restart auxin-server
```

---

## Nginx Reverse Proxy

### Configuration

Create `/etc/nginx/sites-available/auxin`:

```nginx
# See nginx.conf.example for full configuration
upstream auxin_backend {
    server 127.0.0.1:3000;
    keepalive 32;
}

server {
    listen 80;
    listen [::]:80;
    server_name auxin.yourdomain.com;

    # Redirect HTTP to HTTPS (after SSL is set up)
    # return 301 https://$server_name$request_uri;

    # Large file uploads
    client_max_body_size 10G;
    client_body_timeout 300s;

    # WebSocket support
    location /ws {
        proxy_pass http://auxin_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;
    }

    # API and frontend
    location / {
        proxy_pass http://auxin_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts for large uploads
        proxy_connect_timeout 600s;
        proxy_send_timeout 600s;
        proxy_read_timeout 600s;
    }

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;

    # Logging
    access_log /var/log/nginx/auxin-access.log;
    error_log /var/log/nginx/auxin-error.log;
}
```

### Enable Site

```bash
# Enable site
sudo ln -s /etc/nginx/sites-available/auxin /etc/nginx/sites-enabled/

# Test configuration
sudo nginx -t

# Reload Nginx
sudo systemctl reload nginx
```

---

## SSL/TLS with Let's Encrypt

### Obtain Certificate

```bash
# Stop Nginx temporarily
sudo systemctl stop nginx

# Obtain certificate (standalone mode)
sudo certbot certonly --standalone \
  -d auxin.yourdomain.com \
  --agree-tos \
  --email admin@yourdomain.com \
  --non-interactive

# Start Nginx
sudo systemctl start nginx
```

### Configure HTTPS

Update `/etc/nginx/sites-available/auxin`:

```nginx
server {
    listen 80;
    listen [::]:80;
    server_name auxin.yourdomain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name auxin.yourdomain.com;

    # SSL certificates
    ssl_certificate /etc/letsencrypt/live/auxin.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/auxin.yourdomain.com/privkey.pem;

    # SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384';
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # HSTS (optional but recommended)
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    # ... rest of configuration from HTTP section
}
```

### Auto-Renewal

Certbot automatically sets up renewal. Verify:

```bash
# Test renewal
sudo certbot renew --dry-run

# Check renewal timer
sudo systemctl status certbot.timer
```

---

## Systemd Service

For non-Docker deployments, use systemd:

Create `/etc/systemd/system/auxin-server.service`:

```ini
# See auxin-server.service for full configuration
[Unit]
Description=Auxin Server
After=network.target

[Service]
Type=simple
User=auxin
WorkingDirectory=/opt/auxin/auxin-server
Environment="RUST_LOG=info"
ExecStart=/opt/auxin/target/release/auxin-server
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

### Enable and Start

```bash
sudo systemctl daemon-reload
sudo systemctl enable auxin-server
sudo systemctl start auxin-server
sudo systemctl status auxin-server
```

---

## Backup and Restore

### What to Back Up

1. **Repository data**: `/var/oxen/data` (or Docker volume)
2. **Configuration**: `config.toml`, `.env`
3. **User database**: SQLite file if using web-ui feature
4. **Nginx configuration**: `/etc/nginx/sites-available/auxin`

### Automated Backups

Create `/usr/local/bin/backup-auxin.sh`:

```bash
#!/bin/bash
# See backup-auxin.sh for full script

BACKUP_DIR="/backup/auxin"
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_PATH="$BACKUP_DIR/auxin-backup-$DATE"

# Create backup directory
mkdir -p "$BACKUP_PATH"

# Backup repository data
docker run --rm \
  -v auxin_auxin-data:/data \
  -v "$BACKUP_PATH":/backup \
  alpine tar czf /backup/repositories.tar.gz -C /data .

# Backup configuration
cp /opt/auxin/auxin-server/config.toml "$BACKUP_PATH/"
cp /opt/auxin/auxin-server/.env "$BACKUP_PATH/"

# Encrypt backup (optional)
tar czf - "$BACKUP_PATH" | \
  openssl enc -aes-256-cbc -salt -pbkdf2 -out "$BACKUP_PATH.tar.gz.enc"

# Upload to S3 (optional)
# aws s3 cp "$BACKUP_PATH.tar.gz.enc" s3://your-bucket/auxin-backups/

# Cleanup old backups (keep last 30 days)
find "$BACKUP_DIR" -name "auxin-backup-*" -mtime +30 -delete

echo "Backup completed: $BACKUP_PATH"
```

### Schedule Backups

```bash
# Make executable
sudo chmod +x /usr/local/bin/backup-auxin.sh

# Add to crontab (daily at 2 AM)
sudo crontab -e
# Add line:
0 2 * * * /usr/local/bin/backup-auxin.sh
```

### Restore from Backup

```bash
# Stop server
docker compose down

# Decrypt backup (if encrypted)
openssl enc -d -aes-256-cbc -pbkdf2 \
  -in backup.tar.gz.enc \
  -out backup.tar.gz

# Extract
tar xzf backup.tar.gz

# Restore data volume
docker run --rm \
  -v auxin_auxin-data:/data \
  -v $(pwd)/backup:/backup \
  alpine sh -c "rm -rf /data/* && tar xzf /backup/repositories.tar.gz -C /data"

# Restore configuration
cp backup/config.toml /opt/auxin/auxin-server/
cp backup/.env /opt/auxin/auxin-server/

# Start server
docker compose up -d
```

---

## Monitoring

### Health Checks

```bash
# Manual check
curl https://auxin.yourdomain.com/health

# Uptime monitoring (external services)
# - UptimeRobot: https://uptimerobot.com
# - Pingdom: https://www.pingdom.com
# - StatusCake: https://www.statuscake.com
```

### Prometheus + Grafana

See [prometheus.yml](./prometheus.yml) and [grafana-dashboard.json](./grafana-dashboard.json) for full configuration.

**Quick setup**:

```bash
# Add to docker-compose.prod.yml:
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    ports:
      - "127.0.0.1:9090:9090"

  grafana:
    image: grafana/grafana:latest
    volumes:
      - grafana-data:/var/lib/grafana
    ports:
      - "127.0.0.1:3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=changeme

volumes:
  prometheus-data:
  grafana-data:
```

### Logging

**View logs**:
```bash
# Docker logs
docker compose logs -f auxin-server

# Nginx logs
sudo tail -f /var/log/nginx/auxin-access.log
sudo tail -f /var/log/nginx/auxin-error.log

# System logs
journalctl -u auxin-server -f
```

**Centralized logging** (optional):
- ELK Stack (Elasticsearch, Logstash, Kibana)
- Loki + Grafana
- Cloud logging (AWS CloudWatch, Google Cloud Logging)

---

## Security Hardening

### Firewall

```bash
# UFW (Uncomplicated Firewall)
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow http
sudo ufw allow https
sudo ufw enable
```

### Fail2ban

Protect against brute force attacks:

```bash
# Install
sudo apt install fail2ban

# Configure
sudo cp /etc/fail2ban/jail.conf /etc/fail2ban/jail.local
sudo nano /etc/fail2ban/jail.local

# Add Nginx protection
[nginx-limit-req]
enabled = true
port = http,https
logpath = /var/log/nginx/*error.log

# Restart
sudo systemctl restart fail2ban
```

### Docker Security

```bash
# Run as non-root user (already configured in Dockerfile)
# Scan images for vulnerabilities
docker scan auxin-server:latest

# Limit container resources
# Add to docker-compose.prod.yml:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          memory: 2G
```

### Authentication Best Practices

1. **Strong secrets**: Use `openssl rand -base64 32` for `auth_token_secret`
2. **Token expiration**: Set appropriate expiry (default: 24 hours)
3. **HTTPS only**: Never expose HTTP in production
4. **Rate limiting**: Configure Nginx `limit_req` module

---

## Troubleshooting

### Server Won't Start

```bash
# Check logs
docker compose logs auxin-server

# Common issues:
# 1. Port already in use
sudo lsof -i :3000

# 2. Permission issues
sudo chown -R 1000:1000 /var/oxen/data

# 3. Configuration errors
docker compose config
```

### High Memory Usage

```bash
# Check container stats
docker stats auxin-server

# Adjust memory limits in docker-compose.yml
# Optimize Rust build (release mode already uses optimizations)
```

### Slow Performance

1. **Enable Redis locks** for distributed locking:
   ```toml
   [server]
   enable_redis_locks = true
   redis_url = "redis://redis:6379"
   ```

2. **Use SSD** for repository data
3. **Increase resources** (CPU, RAM)
4. **Monitor with Prometheus** to identify bottlenecks

### SSL Certificate Issues

```bash
# Check certificate expiry
sudo certbot certificates

# Force renewal
sudo certbot renew --force-renewal

# Check Nginx SSL config
sudo nginx -t
```

---

## Maintenance

### Updates

```bash
# Pull latest code
cd /opt/auxin
sudo git pull

# Rebuild
cd auxin-server
docker compose build

# Restart with new image
docker compose down
docker compose up -d

# Verify
curl https://auxin.yourdomain.com/health
```

### Database Maintenance

If using web-ui with PostgreSQL:

```bash
# Vacuum database
docker exec -it auxin-postgres psql -U auxin -c "VACUUM ANALYZE;"

# Backup database
docker exec auxin-postgres pg_dump -U auxin auxin > backup.sql
```

---

## Production Checklist

Before going live:

- [ ] Strong `auth_token_secret` configured
- [ ] SSL/TLS certificate installed and auto-renewal working
- [ ] Nginx reverse proxy configured with security headers
- [ ] Firewall configured (UFW)
- [ ] Fail2ban enabled
- [ ] Automated backups scheduled
- [ ] Monitoring (Prometheus/Grafana or external) configured
- [ ] Log rotation configured
- [ ] DNS records properly configured
- [ ] Health check endpoint verified
- [ ] Disk space alerts configured
- [ ] Documentation for team members
- [ ] Disaster recovery plan documented
- [ ] Contact information for emergencies

---

## Support

- **Documentation**: [API Docs](../api/README.md), [Configuration Guide](../../CONFIGURATION.md)
- **GitHub Issues**: https://github.com/jbacus/auxin/issues
- **Community**: (Add Discord/Slack link if available)

---

*Last updated: 2025-11-22*
