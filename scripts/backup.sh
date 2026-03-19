#!/bin/bash
# fluxgate_db_backup.sh
echo "Initiating Automated DB Backup..."
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_DIR="/var/backups/fluxgate"
mkdir -p "$BACKUP_DIR"

DB_URL=${DATABASE_URL:-"postgres://postgres:password@localhost:5432/fluxgate_db"}
pg_dump "$DB_URL" | gzip > "$BACKUP_DIR/fluxgate_backup_$TIMESTAMP.sql.gz"

echo "Backup completed: fluxgate_backup_$TIMESTAMP.sql.gz"
# Optional: Sync to S3
# aws s3 cp "$BACKUP_DIR/fluxgate_backup_$TIMESTAMP.sql.gz" s3://fluxgate-backups/
