#!/bin/bash
#
# Auxin Server Backup Script
#
# This script backs up:
# - Repository data (Docker volume)
# - Configuration files
# - Optional: Encrypt and upload to S3
#
# Usage: ./backup-auxin.sh
# Schedule with cron: 0 2 * * * /usr/local/bin/backup-auxin.sh

set -e

# Configuration
BACKUP_DIR="/backup/auxin"
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_PATH="$BACKUP_DIR/auxin-backup-$DATE"
DOCKER_VOLUME="auxin_auxin-data"
CONFIG_DIR="/opt/auxin/auxin-server"

# Optional: Encryption password (use environment variable for security)
ENCRYPTION_PASSWORD="${BACKUP_ENCRYPTION_PASSWORD:-changeme}"

# Optional: S3 bucket for offsite backups
S3_BUCKET="${BACKUP_S3_BUCKET:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
    exit 1
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Create backup directory
log "Creating backup directory: $BACKUP_PATH"
mkdir -p "$BACKUP_PATH" || error "Failed to create backup directory"

# Check if Docker volume exists
if ! docker volume inspect "$DOCKER_VOLUME" >/dev/null 2>&1; then
    error "Docker volume '$DOCKER_VOLUME' not found"
fi

# Backup repository data
log "Backing up repository data from Docker volume..."
docker run --rm \
    -v "$DOCKER_VOLUME":/data:ro \
    -v "$BACKUP_PATH":/backup \
    alpine tar czf /backup/repositories.tar.gz -C /data . \
    || error "Failed to backup repository data"

# Get size of backup
REPO_SIZE=$(du -h "$BACKUP_PATH/repositories.tar.gz" | cut -f1)
log "Repository data backed up ($REPO_SIZE)"

# Backup configuration files
log "Backing up configuration files..."
if [ -f "$CONFIG_DIR/config.toml" ]; then
    cp "$CONFIG_DIR/config.toml" "$BACKUP_PATH/" || warn "Failed to copy config.toml"
fi

if [ -f "$CONFIG_DIR/.env" ]; then
    cp "$CONFIG_DIR/.env" "$BACKUP_PATH/" || warn "Failed to copy .env"
fi

if [ -f "$CONFIG_DIR/docker-compose.yml" ]; then
    cp "$CONFIG_DIR/docker-compose.yml" "$BACKUP_PATH/" || warn "Failed to copy docker-compose.yml"
fi

# Backup Nginx configuration
if [ -f "/etc/nginx/sites-available/auxin" ]; then
    cp "/etc/nginx/sites-available/auxin" "$BACKUP_PATH/nginx-auxin.conf" || warn "Failed to copy Nginx config"
fi

# Create backup metadata
cat > "$BACKUP_PATH/backup-info.txt" <<EOF
Backup Date: $(date)
Hostname: $(hostname)
Docker Volume: $DOCKER_VOLUME
Repository Size: $REPO_SIZE
Backup Path: $BACKUP_PATH
EOF

log "Backup metadata created"

# Create compressed archive
log "Creating compressed archive..."
cd "$BACKUP_DIR"
tar czf "auxin-backup-$DATE.tar.gz" "auxin-backup-$DATE" \
    || error "Failed to create compressed archive"

ARCHIVE_SIZE=$(du -h "auxin-backup-$DATE.tar.gz" | cut -f1)
log "Compressed archive created ($ARCHIVE_SIZE)"

# Encrypt backup (optional but recommended)
if [ -n "$ENCRYPTION_PASSWORD" ] && [ "$ENCRYPTION_PASSWORD" != "changeme" ]; then
    log "Encrypting backup..."
    openssl enc -aes-256-cbc -salt -pbkdf2 -pass pass:"$ENCRYPTION_PASSWORD" \
        -in "auxin-backup-$DATE.tar.gz" \
        -out "auxin-backup-$DATE.tar.gz.enc" \
        || error "Failed to encrypt backup"

    ENCRYPTED_SIZE=$(du -h "auxin-backup-$DATE.tar.gz.enc" | cut -f1)
    log "Backup encrypted ($ENCRYPTED_SIZE)"

    # Remove unencrypted archive
    rm "auxin-backup-$DATE.tar.gz"
    FINAL_BACKUP="auxin-backup-$DATE.tar.gz.enc"
else
    warn "Backup not encrypted (set BACKUP_ENCRYPTION_PASSWORD environment variable)"
    FINAL_BACKUP="auxin-backup-$DATE.tar.gz"
fi

# Upload to S3 (optional)
if [ -n "$S3_BUCKET" ]; then
    log "Uploading to S3: s3://$S3_BUCKET/auxin-backups/"
    if command -v aws >/dev/null 2>&1; then
        aws s3 cp "$FINAL_BACKUP" "s3://$S3_BUCKET/auxin-backups/" \
            || warn "Failed to upload to S3"
        log "Backup uploaded to S3"
    else
        warn "AWS CLI not found - skipping S3 upload"
    fi
fi

# Cleanup old backups (keep last 30 days)
log "Cleaning up old backups (keeping last 30 days)..."
find "$BACKUP_DIR" -name "auxin-backup-*.tar.gz*" -mtime +30 -delete
find "$BACKUP_DIR" -name "auxin-backup-2*" -type d -mtime +30 -exec rm -rf {} + 2>/dev/null || true

# Remove temporary backup directory
rm -rf "$BACKUP_PATH"

log "Backup completed successfully!"
log "Backup file: $BACKUP_DIR/$FINAL_BACKUP"
log "To restore, run: ./restore-auxin.sh $FINAL_BACKUP"

# Send notification (optional)
# if [ -n "$SLACK_WEBHOOK_URL" ]; then
#     curl -X POST "$SLACK_WEBHOOK_URL" \
#         -H 'Content-Type: application/json' \
#         -d "{\"text\":\"✅ Auxin backup completed: $FINAL_BACKUP ($ENCRYPTED_SIZE)\"}"
# fi

exit 0
