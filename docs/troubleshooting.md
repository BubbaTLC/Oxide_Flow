# Oxide Flow State Management Troubleshooting Guide

## Overview

This guide provides step-by-step solutions for common state management issues, diagnostic procedures, and recovery strategies for Oxide Flow's production state management system.

## Quick Diagnostic Commands

### Health Check
```bash
# Overall system health
oxide_flow state diagnostics

# Specific pipeline health
oxide_flow state validate <pipeline>

# Backend integrity check
oxide_flow state verify-integrity

# Performance metrics
oxide_flow state diagnostics --verbose
```

### State Information
```bash
# List all states
oxide_flow state list

# Show specific state details
oxide_flow state show <pipeline>

# Check active workers
oxide_flow worker list
```

## Common Issues & Solutions

### 1. State File Corruption

#### Symptoms
- Error: "State file corrupted"
- Error: "Deserialization failed"
- Pipeline fails to load previous state

#### Diagnosis
```bash
# Check specific pipeline
oxide_flow state validate <pipeline>

# Check all pipelines
oxide_flow state verify-integrity
```

#### Solutions

**Option 1: Automatic Repair**
```bash
# Attempt automatic repair (creates backup first)
oxide_flow state repair <pipeline>

# Validate the repair
oxide_flow state validate <pipeline>
```

**Option 2: Restore from Backup**
```bash
# List available backups
oxide_flow state list-backups <pipeline>

# Restore from specific backup
oxide_flow state restore <pipeline> --backup <backup-id>

# Restore from latest backup
oxide_flow state restore <pipeline> --backup latest
```

**Option 3: Manual Recovery**
```bash
# Export corrupted state for analysis
oxide_flow state export <pipeline> --format json --output corrupted_state.json

# Edit the JSON manually to fix issues
# Re-import the corrected state
oxide_flow state import <pipeline> --file corrected_state.json
```

### 2. Lock Timeout Issues

#### Symptoms
- Error: "Lock acquisition timeout"
- Error: "Lock already held by worker"
- Pipeline hangs waiting for locks

#### Diagnosis
```bash
# Check for stuck locks
oxide_flow worker list

# Check specific pipeline locks
oxide_flow state show <pipeline> | grep -i lock
```

#### Solutions

**Option 1: Wait for Lock Expiration**
```bash
# Check lock timeout configuration
grep lock_timeout oxiflow.yaml

# Locks automatically expire after timeout period
```

**Option 2: Stop Specific Worker**
```bash
# Stop the worker holding the lock
oxide_flow worker stop <worker-id>

# Verify lock is released
oxide_flow worker list
```

**Option 3: Force Release Lock**
```bash
# Force release lock (use with caution)
oxide_flow state unlock <pipeline> --force

# Verify no data corruption occurred
oxide_flow state validate <pipeline>
```

### 3. Performance Issues

#### Symptoms
- Slow state operations
- High memory usage
- Low cache hit rates

#### Diagnosis
```bash
# Get detailed performance metrics
oxide_flow state diagnostics --verbose

# Check specific metrics
oxide_flow state diagnostics | grep -E "(hit_rate|avg_.*_time_ms)"
```

#### Solutions

**High Read Times (>100ms)**
```bash
# Check cache hit rate
oxide_flow state diagnostics | grep cache_hit_rate

# If low (<50%), increase cache size in oxiflow.yaml:
state:
  backend_config:
    cache_size: 200  # Increase from default 100
```

**High Write Times (>200ms)**
```bash
# Check if atomic writes are enabled
grep atomic_writes oxiflow.yaml

# Temporarily disable for testing (not recommended for production)
state:
  backend_config:
    atomic_writes: false
```

**Memory Issues**
```bash
# Clear cache to free memory
oxide_flow state clear-cache

# Reduce cache size
state:
  backend_config:
    cache_size: 50  # Reduce from current size
```

### 4. Disk Space Issues

#### Symptoms
- Error: "No space left on device"
- Error: "Insufficient disk space"
- Write operations failing

#### Diagnosis
```bash
# Check disk usage
df -h .oxiflow/

# Check state directory size
du -sh .oxiflow/state/

# Check backup directory size
du -sh .oxiflow/state/backups/
```

#### Solutions

**Immediate Cleanup**
```bash
# Clean up old states (older than 7 days)
oxide_flow state cleanup --older-than 7d

# Clean up old backups
oxide_flow state cleanup --backups --older-than 3d

# Clean up all stale states
oxide_flow state cleanup --stale
```

**Configuration Adjustments**
```yaml
# Reduce backup retention in oxiflow.yaml
state:
  backend_config:
    backup_retention: "3d"  # Reduce from 7d

  cleanup_interval: "6h"    # More frequent cleanup
```

### 5. Permission Issues

#### Symptoms
- Error: "Permission denied"
- Error: "Access denied"
- Cannot read/write state files

#### Diagnosis
```bash
# Check file permissions
ls -la .oxiflow/state/

# Check directory permissions
ls -ld .oxiflow/state/
```

#### Solutions

**Fix Permissions**
```bash
# Fix directory permissions
chmod 755 .oxiflow/state/
chmod 755 .oxiflow/state/states/
chmod 755 .oxiflow/state/locks/
chmod 755 .oxiflow/state/backups/

# Fix file permissions
chmod 644 .oxiflow/state/states/*
chmod 644 .oxiflow/state/locks/*
chmod 644 .oxiflow/state/backups/*/*
```

**SELinux Issues (if applicable)**
```bash
# Check SELinux context
ls -Z .oxiflow/state/

# Restore SELinux context
restorecon -R .oxiflow/state/
```

### 6. Backup and Restore Issues

#### Symptoms
- Backup creation fails
- Restore operation fails
- Missing backups

#### Diagnosis
```bash
# Check backup configuration
grep -A5 backup_enabled oxiflow.yaml

# List existing backups
oxide_flow state list-backups <pipeline>

# Check backup directory
ls -la .oxiflow/state/backups/
```

#### Solutions

**Enable Backups**
```yaml
# In oxiflow.yaml
state:
  backend_config:
    backup_enabled: true
    backup_retention: "7d"
```

**Manual Backup Creation**
```bash
# Create manual backup
oxide_flow state backup <pipeline>

# Verify backup was created
oxide_flow state list-backups <pipeline>
```

**Restore Issues**
```bash
# Try restoring from different backup
oxide_flow state list-backups <pipeline>
oxide_flow state restore <pipeline> --backup <different-backup-id>

# Check backup file integrity
oxide_flow state validate-backup <pipeline> <backup-id>
```

## Advanced Diagnostics

### State File Analysis

```bash
# Export state for manual inspection
oxide_flow state export <pipeline> --format json --output debug_state.json

# Validate JSON structure
jq '.' debug_state.json

# Check specific fields
jq '.pipeline_id, .version, .status' debug_state.json
```

### Lock Analysis

```bash
# Check lock files directly
ls -la .oxiflow/state/locks/

# Inspect lock file contents
cat .oxiflow/state/locks/<pipeline>.lock | jq '.'

# Check lock expiration times
jq '.expires_at' .oxiflow/state/locks/<pipeline>.lock
```

### Performance Profiling

```bash
# Run performance benchmark
oxide_flow state benchmark --operations 100 --pipeline test_perf

# Monitor real-time metrics during operations
watch "oxide_flow state diagnostics | grep -E '(hit_rate|avg_.*_time_ms)'"

# Profile specific operations
time oxide_flow state show <pipeline>
time oxide_flow state export <pipeline> --format json
```

## Recovery Procedures

### Complete State Directory Loss

1. **Stop all pipeline operations**
2. **Restore from external backups** (if available)
3. **Reinitialize state directory**:
   ```bash
   mkdir -p .oxiflow/state/{states,locks,backups}
   chmod 755 .oxiflow/state/{states,locks,backups}
   ```
4. **Restart pipelines** (they will create new state)

### Partial State Corruption

1. **Identify affected pipelines**:
   ```bash
   oxide_flow state verify-integrity
   ```

2. **For each corrupted pipeline**:
   ```bash
   # Create emergency backup
   cp .oxiflow/state/states/<pipeline>.json emergency_backup.json

   # Attempt repair
   oxide_flow state repair <pipeline>

   # Validate repair
   oxide_flow state validate <pipeline>
   ```

3. **If repair fails**:
   ```bash
   # Restore from backup
   oxide_flow state restore <pipeline> --backup latest
   ```

### Backend Failure Recovery

1. **Switch to memory backend temporarily**:
   ```yaml
   # In oxiflow.yaml
   state:
     backend: "memory"
   ```

2. **Fix file backend issues**
3. **Export states from memory**:
   ```bash
   for pipeline in $(oxide_flow state list); do
     oxide_flow state export $pipeline --format json --output ${pipeline}_memory.json
   done
   ```

4. **Switch back to file backend**:
   ```yaml
   state:
     backend: "file"
   ```

5. **Import states back**:
   ```bash
   for file in *_memory.json; do
     pipeline=$(basename $file _memory.json)
     oxide_flow state import $pipeline --file $file
   done
   ```

## Monitoring & Alerting

### Key Metrics to Monitor

1. **Error Rate**: `errors_per_minute < 1`
2. **Cache Hit Rate**: `cache_hit_rate > 0.8`
3. **Average Read Time**: `avg_read_time_ms < 50`
4. **Average Write Time**: `avg_write_time_ms < 100`
5. **Disk Usage**: `state_directory_size < threshold`

### Alerting Commands

```bash
# Check error rate
oxide_flow state diagnostics | jq '.performance_metrics.error_rate'

# Check if cache hit rate is low
cache_hit_rate=$(oxide_flow state diagnostics | jq '.performance_metrics.cache_hit_rate')
if (( $(echo "$cache_hit_rate < 0.5" | bc -l) )); then
  echo "ALERT: Low cache hit rate: $cache_hit_rate"
fi

# Check disk usage
disk_usage=$(du -s .oxiflow/state/ | cut -f1)
if [ $disk_usage -gt 1000000 ]; then  # 1GB in KB
  echo "ALERT: High disk usage: ${disk_usage}KB"
fi
```

### Health Check Script

```bash
#!/bin/bash
# health_check.sh

echo "=== Oxide Flow State Management Health Check ==="

# Check if state management is enabled
if ! grep -q "enabled: true" oxiflow.yaml; then
  echo "WARNING: State management not enabled"
  exit 1
fi

# Check state directory exists and is writable
if [ ! -w .oxiflow/state/ ]; then
  echo "ERROR: State directory not writable"
  exit 1
fi

# Check for corrupted states
corrupted=$(oxide_flow state verify-integrity --json | jq '.corrupted_files | length')
if [ "$corrupted" -gt 0 ]; then
  echo "ERROR: $corrupted corrupted state files found"
  exit 1
fi

# Check performance metrics
hit_rate=$(oxide_flow state diagnostics --json | jq '.performance_metrics.cache_hit_rate // 0')
if (( $(echo "$hit_rate < 0.5" | bc -l) )); then
  echo "WARNING: Low cache hit rate: $hit_rate"
fi

echo "OK: State management healthy"
```

## Prevention Strategies

### Configuration Best Practices

```yaml
# Production-ready configuration
state:
  enabled: true
  backend: "file"
  backend_config:
    state_dir: ".oxiflow/state"
    lock_timeout: "30s"
    backup_enabled: true
    backup_retention: "7d"
    atomic_writes: true
    cache_size: 200

  heartbeat_interval: "10s"
  checkpoint_interval: "30s"
  cleanup_interval: "1h"
```

### Maintenance Schedule

- **Daily**: Check disk usage and error rates
- **Weekly**: Run integrity verification
- **Monthly**: Review and clean up old backups
- **Quarterly**: Performance review and optimization

### Backup Strategy

1. **Automatic backups**: Before any repair operation
2. **Scheduled backups**: Daily at low-traffic times
3. **External backups**: Weekly backup to external storage
4. **Disaster recovery**: Monthly full system backup

This troubleshooting guide should help you quickly identify and resolve common state management issues. For issues not covered here, enable debug logging and examine the detailed error messages and stack traces.
