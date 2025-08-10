# Oxide Flow State Management

## Overview

Oxide Flow's state management system provides robust, production-ready pipeline state tracking with file-based persistence, comprehensive error recovery, and performance optimization features. This system tracks pipeline execution progress, handles failures gracefully, and provides the foundation for future distributed deployments.

## Features

### Core Capabilities
- **Pipeline State Tracking**: Complete execution state with step-level progress
- **Persistence**: File-based storage with atomic operations and integrity guarantees
- **Error Recovery**: Automatic corruption detection, backup/restore, and repair
- **Performance Optimization**: Intelligent caching, metrics collection, and I/O optimization
- **CLI Integration**: Comprehensive command-line tools for state management

### Production Hardening
- **Data Integrity**: Checksums, validation, and corruption detection
- **Backup & Restore**: Automatic backups with point-in-time recovery
- **Self-Healing**: Automatic repair of common state corruption issues
- **Performance Monitoring**: Real-time metrics and diagnostics
- **Graceful Degradation**: Fallback mechanisms for backend failures

## Architecture

### State Structure

```rust
pub struct PipelineState {
    // Identity and versioning
    pub pipeline_id: String,
    pub run_id: String,
    pub version: u64,

    // Progress tracking
    pub last_processed_id: String,
    pub batch_number: u64,
    pub records_processed: u64,
    pub records_failed: u64,
    pub data_size_processed: u64,

    // Execution state
    pub current_step: String,
    pub step_states: HashMap<String, StepState>,
    pub status: PipelineStatus,

    // Timing and metadata
    pub started_at: DateTime<Utc>,
    pub last_success_timestamp: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,

    // Error tracking
    pub errors: Vec<ErrorRecord>,
    pub retry_count: u64,

    // Worker coordination
    pub worker_id: Option<String>,
    pub last_heartbeat: DateTime<Utc>,
}
```

### Backend Architecture

The state management system uses a pluggable backend architecture:

```
StateBackend Trait
├── FileBackend (Production)
│   ├── File-based persistence
│   ├── Atomic operations
│   ├── Performance caching
│   └── Error recovery
└── MemoryBackend (Testing)
    ├── In-memory storage
    ├── Fast operations
    └── Development use
```

## Configuration

### Project Configuration

Add state management configuration to your `oxiflow.yaml`:

```yaml
state:
  enabled: true
  backend: "file"
  backend_config:
    state_dir: ".oxiflow/state"
    lock_timeout: "30s"
    backup_enabled: true
    backup_retention: "7d"

  heartbeat_interval: "10s"
  checkpoint_interval: "30s"
  cleanup_interval: "1h"
```

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `enabled` | Enable state tracking | `true` |
| `backend` | Backend type (`file` or `memory`) | `file` |
| `state_dir` | Directory for state files | `.oxiflow/state` |
| `lock_timeout` | Lock acquisition timeout | `30s` |
| `backup_enabled` | Enable automatic backups | `true` |
| `backup_retention` | How long to keep backups | `7d` |
| `heartbeat_interval` | Worker heartbeat frequency | `10s` |
| `checkpoint_interval` | State checkpoint frequency | `30s` |
| `cleanup_interval` | Cleanup operation frequency | `1h` |

## CLI Commands

### State Management

```bash
# View pipeline state
oxide_flow state show <pipeline>

# List all pipeline states
oxide_flow state list
oxide_flow state list --active

# Export state data
oxide_flow state export <pipeline> --format json
oxide_flow state export <pipeline> --format yaml

# Import state data
oxide_flow state import <pipeline> --file state_backup.json

# Clean up old states
oxide_flow state cleanup --stale
oxide_flow state cleanup --older-than 7d
```

### Worker Management

```bash
# List active workers
oxide_flow worker list
oxide_flow worker list --pipeline <pipeline>

# Stop a worker
oxide_flow worker stop <worker-id>

# Get worker details
oxide_flow worker show <worker-id>
```

### Maintenance Operations

```bash
# Validate state integrity
oxide_flow state validate <pipeline>

# Create manual backup
oxide_flow state backup <pipeline>

# Restore from backup
oxide_flow state restore <pipeline> --backup <backup-id>

# Repair corrupted state
oxide_flow state repair <pipeline>

# Get diagnostics
oxide_flow state diagnostics
```

## Performance Features

### Intelligent Caching

The file backend includes an intelligent LRU cache that:
- Reduces disk I/O for frequently accessed states
- Automatically evicts least recently used entries
- Provides cache hit rate monitoring
- Can be configured for different cache sizes

### Performance Metrics

Real-time performance monitoring includes:
- Read/write operation timing
- Serialization/deserialization performance
- Cache hit rates and efficiency
- I/O throughput metrics
- Storage utilization

### I/O Optimization

- **Atomic Writes**: Prevent corruption during write operations
- **Efficient Serialization**: Optimized JSON/YAML serialization
- **Batch Operations**: Grouped operations for better performance
- **Lock Optimization**: Minimal lock contention with timeout handling

## Error Handling & Recovery

### Automatic Detection

The system automatically detects:
- File corruption (checksums, invalid data)
- Missing state files
- Lock conflicts and timeouts
- Invalid state structure
- Timestamp inconsistencies

### Recovery Mechanisms

1. **Validation & Repair**
   ```bash
   oxide_flow state validate pipeline_name
   oxide_flow state repair pipeline_name
   ```

2. **Backup & Restore**
   ```bash
   oxide_flow state backup pipeline_name
   oxide_flow state restore pipeline_name --backup backup_id
   ```

3. **Integrity Verification**
   ```bash
   oxide_flow state verify-integrity
   ```

### Recovery Strategies

| Issue | Detection | Recovery Action |
|-------|-----------|-----------------|
| File Corruption | Checksum mismatch | Restore from latest backup |
| Invalid Timestamps | Validation check | Automatic timestamp correction |
| Missing Fields | Schema validation | Field reconstruction with defaults |
| Lock Conflicts | Timeout detection | Force lock release with confirmation |
| Disk Space | Write failure | Cleanup old states and backups |

## Production Deployment

### File System Requirements

- **Permissions**: Read/write access to state directory
- **Disk Space**: Plan for 1-10MB per active pipeline
- **Backup Storage**: Additional 2-3x space for backup retention
- **File System**: POSIX-compliant file system with atomic rename support

### Performance Tuning

```yaml
# High-performance configuration
state:
  backend_config:
    cache_size: 500          # Larger cache for frequently accessed states
    atomic_writes: true      # Ensure data integrity
    backup_enabled: true     # Regular backups

  heartbeat_interval: "5s"   # More frequent heartbeats for monitoring
  checkpoint_interval: "15s" # More frequent checkpoints
```

### Monitoring Setup

Monitor these key metrics:
- Cache hit rate (target: >80%)
- Average read time (target: <50ms)
- Average write time (target: <100ms)
- Error rate (target: <1%)
- Storage growth rate

### Backup Strategy

1. **Automatic Backups**: Created before any repair operation
2. **Scheduled Backups**: Regular snapshots based on configuration
3. **Manual Backups**: On-demand backups for critical operations
4. **Retention Policy**: Automatic cleanup based on age and count

## Troubleshooting

### Common Issues

#### "State file corrupted" Errors

```bash
# Diagnose the issue
oxide_flow state validate pipeline_name

# Attempt automatic repair
oxide_flow state repair pipeline_name

# Manual restore if needed
oxide_flow state restore pipeline_name --backup latest
```

#### High Memory Usage

```bash
# Check cache statistics
oxide_flow state diagnostics

# Clear cache if needed (forces reload from disk)
oxide_flow state clear-cache
```

#### Lock Timeout Issues

```bash
# Check for stuck locks
oxide_flow worker list

# Force release stuck locks
oxide_flow worker stop worker_id

# Or force release all locks for a pipeline
oxide_flow state unlock pipeline_name --force
```

#### Performance Issues

```bash
# Get detailed performance metrics
oxide_flow state diagnostics --verbose

# Check for I/O bottlenecks
oxide_flow state benchmark --operations 100
```

### Log Analysis

State management operations are logged with structured data:

```
INFO  [oxide_flow::state] State loaded from cache pipeline=batch_demo duration_ms=0.5
INFO  [oxide_flow::state] State saved to disk pipeline=batch_demo duration_ms=15.2 bytes=2048
WARN  [oxide_flow::state] Cache hit rate low rate=0.45 recommendations="increase cache size"
ERROR [oxide_flow::state] State corruption detected pipeline=demo_pipeline action="automatic backup created"
```

### Recovery Procedures

#### Complete State Loss

1. **Stop all workers** accessing the affected pipeline
2. **Check for backups**: `oxide_flow state list-backups pipeline_name`
3. **Restore from backup**: `oxide_flow state restore pipeline_name --backup backup_id`
4. **Validate restored state**: `oxide_flow state validate pipeline_name`
5. **Resume operations**

#### Partial Corruption

1. **Create immediate backup**: `oxide_flow state backup pipeline_name`
2. **Run repair**: `oxide_flow state repair pipeline_name`
3. **Validate repair**: `oxide_flow state validate pipeline_name`
4. **Check for data loss**: Compare pre/post repair state

## Migration & Upgrades

### Backend Migration

Future migration to distributed backends will be supported through:
- Configuration-based backend selection
- State export/import tools
- Automated migration utilities
- Zero-downtime migration strategies

### Schema Evolution

State schema changes are handled through:
- Version-aware deserialization
- Automatic field migration
- Backward compatibility guarantees
- Migration validation tools

## Security Considerations

### File Permissions

- State files should be readable/writable only by the pipeline user
- Lock files require appropriate permissions for multi-user scenarios
- Backup files should follow the same permission model

### Data Protection

- State files may contain sensitive pipeline data
- Consider encryption for sensitive deployments
- Implement appropriate access controls
- Regular security audits of state directories

## Best Practices

### Configuration

1. **Enable state tracking** for all production pipelines
2. **Configure appropriate retention** for your use case
3. **Monitor cache hit rates** and adjust cache size accordingly
4. **Set up alerting** for error rates and performance degradation

### Operations

1. **Regular backup verification** - test restore procedures
2. **Monitor disk usage** - implement cleanup policies
3. **Performance baseline** - establish normal operation metrics
4. **Incident response** - have recovery procedures documented

### Development

1. **Use memory backend** for testing and development
2. **Test state recovery** scenarios in development
3. **Validate configuration** before production deployment
4. **Performance testing** with realistic data volumes

## Future Enhancements

The state management system is designed for evolution:

### Planned Features
- **Distributed Backends**: Redis, HTTP, Database backends
- **Advanced Analytics**: State analytics and reporting
- **Orchestrator Integration**: Kubernetes, Docker Swarm support
- **Multi-Region**: Cross-region state synchronization
- **Advanced Monitoring**: Prometheus metrics, health checks

### Extension Points
- Custom backend implementations
- Pluggable serialization formats
- Custom validation rules
- External monitoring integrations

This foundation provides a solid base for these future enhancements while maintaining backward compatibility and production stability.
