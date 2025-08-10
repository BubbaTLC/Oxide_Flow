#!/bin/bash

# Oxide Flow State Management Monitoring Setup
# This script sets up comprehensive monitoring for state management

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_DIR="/var/log/oxiflow"
METRICS_DIR="/var/lib/oxiflow/metrics"

echo "=== Oxide Flow State Management Monitoring Setup ==="

# Create directories
sudo mkdir -p "$LOG_DIR" "$METRICS_DIR"
sudo chown oxiflow:oxiflow "$LOG_DIR" "$METRICS_DIR"

# Health check script
cat > /tmp/oxiflow_health_check.sh << 'EOF'
#!/bin/bash

# Oxide Flow Health Check Script
# Run this regularly to monitor state management health

LOG_FILE="/var/log/oxiflow/health_check.log"
ALERT_THRESHOLD_ERROR_RATE=0.05
ALERT_THRESHOLD_CACHE_HIT_RATE=0.7
ALERT_THRESHOLD_AVG_READ_TIME=100
ALERT_THRESHOLD_DISK_USAGE=85

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') $*" | tee -a "$LOG_FILE"
}

alert() {
    local level="$1"
    local message="$2"
    log "[$level] $message"

    # Send to external monitoring (customize as needed)
    if [ -n "${SLACK_WEBHOOK_URL:-}" ]; then
        curl -X POST -H 'Content-type: application/json' \
            --data "{\"text\":\"Oxide Flow Alert [$level]: $message\"}" \
            "$SLACK_WEBHOOK_URL" || true
    fi
}

# Check if state management is running
if ! pgrep -f "oxide_flow" > /dev/null; then
    alert "CRITICAL" "Oxide Flow process not running"
    exit 1
fi

# Get diagnostics
DIAGNOSTICS=$(oxiflow state diagnostics --json 2>/dev/null || echo '{}')

# Check error rate
ERROR_RATE=$(echo "$DIAGNOSTICS" | jq -r '.performance_metrics.error_rate // 0')
if (( $(echo "$ERROR_RATE > $ALERT_THRESHOLD_ERROR_RATE" | bc -l) )); then
    alert "WARNING" "High error rate: $ERROR_RATE (threshold: $ALERT_THRESHOLD_ERROR_RATE)"
fi

# Check cache hit rate
CACHE_HIT_RATE=$(echo "$DIAGNOSTICS" | jq -r '.performance_metrics.cache_hit_rate // 1')
if (( $(echo "$CACHE_HIT_RATE < $ALERT_THRESHOLD_CACHE_HIT_RATE" | bc -l) )); then
    alert "WARNING" "Low cache hit rate: $CACHE_HIT_RATE (threshold: $ALERT_THRESHOLD_CACHE_HIT_RATE)"
fi

# Check average read time
AVG_READ_TIME=$(echo "$DIAGNOSTICS" | jq -r '.performance_metrics.avg_read_time_ms // 0')
if (( $(echo "$AVG_READ_TIME > $ALERT_THRESHOLD_AVG_READ_TIME" | bc -l) )); then
    alert "WARNING" "High average read time: ${AVG_READ_TIME}ms (threshold: ${ALERT_THRESHOLD_AVG_READ_TIME}ms)"
fi

# Check disk usage
DISK_USAGE=$(df /var/lib/oxiflow/state | awk 'NR==2 {print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt "$ALERT_THRESHOLD_DISK_USAGE" ]; then
    alert "WARNING" "High disk usage: ${DISK_USAGE}% (threshold: ${ALERT_THRESHOLD_DISK_USAGE}%)"
fi

# Check for corrupted states
CORRUPTED_COUNT=$(oxiflow state verify-integrity --json 2>/dev/null | jq -r '.corrupted_files | length' || echo "0")
if [ "$CORRUPTED_COUNT" -gt 0 ]; then
    alert "ERROR" "$CORRUPTED_COUNT corrupted state files detected"
fi

# Check for stuck locks
STUCK_LOCKS=$(oxiflow worker list --json 2>/dev/null | jq -r '[.[] | select(.last_heartbeat < (now - 300))] | length' || echo "0")
if [ "$STUCK_LOCKS" -gt 0 ]; then
    alert "WARNING" "$STUCK_LOCKS potentially stuck locks detected"
fi

log "INFO Health check completed successfully"

# Store metrics for trending
echo "$DIAGNOSTICS" > "/var/lib/oxiflow/metrics/diagnostics_$(date +%s).json"

# Clean up old metrics (keep last 24 hours)
find /var/lib/oxiflow/metrics/ -name "diagnostics_*.json" -mtime +1 -delete
EOF

# Install health check script
sudo cp /tmp/oxiflow_health_check.sh /usr/local/bin/oxiflow_health_check.sh
sudo chmod +x /usr/local/bin/oxiflow_health_check.sh

# Performance monitoring script
cat > /tmp/oxiflow_performance_monitor.sh << 'EOF'
#!/bin/bash

# Oxide Flow Performance Monitor
# Collects detailed performance metrics

METRICS_FILE="/var/lib/oxiflow/metrics/performance_$(date +%Y%m%d).csv"

# Create CSV header if file doesn't exist
if [ ! -f "$METRICS_FILE" ]; then
    echo "timestamp,total_reads,total_writes,cache_hits,cache_misses,cache_hit_rate,avg_read_time_ms,avg_write_time_ms,total_states,storage_used_bytes" > "$METRICS_FILE"
fi

# Collect metrics
DIAGNOSTICS=$(oxiflow state diagnostics --json 2>/dev/null || echo '{}')
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

TOTAL_READS=$(echo "$DIAGNOSTICS" | jq -r '.performance_metrics.total_reads // 0')
TOTAL_WRITES=$(echo "$DIAGNOSTICS" | jq -r '.performance_metrics.total_writes // 0')
CACHE_HITS=$(echo "$DIAGNOSTICS" | jq -r '.performance_metrics.cache_hits // 0')
CACHE_MISSES=$(echo "$DIAGNOSTICS" | jq -r '.performance_metrics.cache_misses // 0')
CACHE_HIT_RATE=$(echo "$DIAGNOSTICS" | jq -r '.performance_metrics.cache_hit_rate // 0')
AVG_READ_TIME=$(echo "$DIAGNOSTICS" | jq -r '.performance_metrics.avg_read_time_ms // 0')
AVG_WRITE_TIME=$(echo "$DIAGNOSTICS" | jq -r '.performance_metrics.avg_write_time_ms // 0')
TOTAL_STATES=$(echo "$DIAGNOSTICS" | jq -r '.total_states // 0')
STORAGE_USED=$(echo "$DIAGNOSTICS" | jq -r '.storage_used_bytes // 0')

# Append to CSV
echo "$TIMESTAMP,$TOTAL_READS,$TOTAL_WRITES,$CACHE_HITS,$CACHE_MISSES,$CACHE_HIT_RATE,$AVG_READ_TIME,$AVG_WRITE_TIME,$TOTAL_STATES,$STORAGE_USED" >> "$METRICS_FILE"

# Clean up old performance files (keep last 30 days)
find /var/lib/oxiflow/metrics/ -name "performance_*.csv" -mtime +30 -delete
EOF

sudo cp /tmp/oxiflow_performance_monitor.sh /usr/local/bin/oxiflow_performance_monitor.sh
sudo chmod +x /usr/local/bin/oxiflow_performance_monitor.sh

# Cleanup script
cat > /tmp/oxiflow_cleanup.sh << 'EOF'
#!/bin/bash

# Oxide Flow Automatic Cleanup
# Performs maintenance tasks

LOG_FILE="/var/log/oxiflow/cleanup.log"

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') $*" | tee -a "$LOG_FILE"
}

log "Starting automatic cleanup"

# Clean up old states (configurable)
CLEANUP_AGE_DAYS=${OXIFLOW_CLEANUP_AGE_DAYS:-30}
log "Cleaning up states older than $CLEANUP_AGE_DAYS days"
oxiflow state cleanup --older-than "${CLEANUP_AGE_DAYS}d" >> "$LOG_FILE" 2>&1

# Clean up old backups
BACKUP_RETENTION_DAYS=${OXIFLOW_BACKUP_RETENTION_DAYS:-14}
log "Cleaning up backups older than $BACKUP_RETENTION_DAYS days"
oxiflow state cleanup --backups --older-than "${BACKUP_RETENTION_DAYS}d" >> "$LOG_FILE" 2>&1

# Clean up stale locks
log "Cleaning up stale locks"
oxiflow state cleanup --stale >> "$LOG_FILE" 2>&1

# Verify integrity after cleanup
log "Running integrity check"
INTEGRITY_RESULT=$(oxiflow state verify-integrity --json 2>/dev/null || echo '{}')
CORRUPTED_COUNT=$(echo "$INTEGRITY_RESULT" | jq -r '.corrupted_files | length' || echo "0")

if [ "$CORRUPTED_COUNT" -gt 0 ]; then
    log "WARNING: $CORRUPTED_COUNT corrupted files found after cleanup"
else
    log "Integrity check passed"
fi

log "Cleanup completed"
EOF

sudo cp /tmp/oxiflow_cleanup.sh /usr/local/bin/oxiflow_cleanup.sh
sudo chmod +x /usr/local/bin/oxiflow_cleanup.sh

# Set up cron jobs
echo "Setting up cron jobs..."

# Health check every 5 minutes
(crontab -l 2>/dev/null || true; echo "*/5 * * * * /usr/local/bin/oxiflow_health_check.sh") | crontab -

# Performance monitoring every minute
(crontab -l 2>/dev/null || true; echo "* * * * * /usr/local/bin/oxiflow_performance_monitor.sh") | crontab -

# Cleanup daily at 2 AM
(crontab -l 2>/dev/null || true; echo "0 2 * * * /usr/local/bin/oxiflow_cleanup.sh") | crontab -

# Log rotation setup
cat > /tmp/oxiflow_logrotate << 'EOF'
/var/log/oxiflow/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 oxiflow oxiflow
    postrotate
        # Send SIGHUP to oxiflow if running
        pkill -HUP -f "oxide_flow" || true
    endscript
}
EOF

sudo cp /tmp/oxiflow_logrotate /etc/logrotate.d/oxiflow

# Create Grafana dashboard configuration (if Grafana is available)
if command -v grafana-cli &> /dev/null; then
    echo "Creating Grafana dashboard..."

    cat > /tmp/oxiflow_dashboard.json << 'EOF'
{
  "dashboard": {
    "title": "Oxide Flow State Management",
    "panels": [
      {
        "title": "Cache Hit Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "oxiflow_cache_hit_rate",
            "legendFormat": "Hit Rate"
          }
        ]
      },
      {
        "title": "Operation Times",
        "type": "graph",
        "targets": [
          {
            "expr": "oxiflow_avg_read_time_ms",
            "legendFormat": "Read Time (ms)"
          },
          {
            "expr": "oxiflow_avg_write_time_ms",
            "legendFormat": "Write Time (ms)"
          }
        ]
      },
      {
        "title": "Storage Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "oxiflow_storage_used_bytes",
            "legendFormat": "Storage Used (bytes)"
          }
        ]
      }
    ]
  }
}
EOF

    # Note: This would need to be imported into Grafana manually
    echo "Grafana dashboard configuration created at /tmp/oxiflow_dashboard.json"
    echo "Import this into Grafana manually"
fi

echo ""
echo "=== Monitoring Setup Complete ==="
echo ""
echo "Installed scripts:"
echo "  - /usr/local/bin/oxiflow_health_check.sh (runs every 5 minutes)"
echo "  - /usr/local/bin/oxiflow_performance_monitor.sh (runs every minute)"
echo "  - /usr/local/bin/oxiflow_cleanup.sh (runs daily at 2 AM)"
echo ""
echo "Log files:"
echo "  - /var/log/oxiflow/health_check.log"
echo "  - /var/log/oxiflow/cleanup.log"
echo ""
echo "Metrics directory:"
echo "  - /var/lib/oxiflow/metrics/"
echo ""
echo "Environment variables for customization:"
echo "  - SLACK_WEBHOOK_URL: Slack notifications"
echo "  - OXIFLOW_CLEANUP_AGE_DAYS: Age threshold for cleanup (default: 30)"
echo "  - OXIFLOW_BACKUP_RETENTION_DAYS: Backup retention (default: 14)"
echo ""
echo "To view current metrics:"
echo "  oxiflow state diagnostics --verbose"
echo ""
echo "To test health check:"
echo "  /usr/local/bin/oxiflow_health_check.sh"

# Clean up temporary files
rm -f /tmp/oxiflow_*.sh /tmp/oxiflow_*.json /tmp/oxiflow_logrotate
