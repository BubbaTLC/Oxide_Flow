# Planned Features for Oxide Flow

## 🔮 Future Enhancements

### Pipeline Performance Estimation
**Status:** Future Consideration
**Priority:** Medium

Add intelligent execution time estimation for pipelines based on:
- Historical performance data tracking actual execution times
- Oxi-specific performance profiles (base overhead, per-byte costs, CPU intensity)
- Data size analysis from file metadata or previous step outputs
- Machine learning models trained on execution patterns

This would provide users with estimated execution times in pipeline listings and detailed performance breakdowns in pipeline info, helping with capacity planning and optimization decisions. The system could learn from actual executions to improve accuracy over time.

**Implementation Approach:**
- Performance cache system storing historical execution data
- Per-Oxi performance profiles with configurable characteristics
- Pipeline-level estimation combining individual Oxi estimates
- Machine learning integration for improved accuracy over time
