---
mode: agent
---

# Implement Plan Phase Prompt

## 🎯 Purpose
This prompt guides the implementation of a specific phase from a plan document, with automatic progress tracking and plan updates.

## 📋 Instructions

### 1. **Phase Analysis**
- Read the specified plan document (typically `docs/current_plan.md`)
- Identify the target phase to implement
- Break down the phase into specific tasks and deliverables
- Assess dependencies and prerequisites

### 2. **Implementation Process**
- Follow the implementation steps defined in the phase
- Create new files as specified in the plan
- Update existing files according to requirements
- Ensure all code compiles and functions correctly
- Write comprehensive tests when applicable

### 3. **Testing & Validation**
- Compile the project to verify no build errors: `cargo build`
- Test new functionality with various inputs/options
- Validate that existing functionality still works
- Run any automated tests: `cargo test`
- Check error handling and edge cases

#### Example Project Testing Commands
For comprehensive testing, use the `example_project` directory:

```bash
# Navigate to example project for testing
cd example_project

# Build the binary first
cargo build --manifest-path ../Cargo.toml

# Test pipeline discovery and listing
../target/debug/oxide_flow pipeline list
../target/debug/oxide_flow pipeline list --verbose

# Test individual pipeline validation
../target/debug/oxide_flow pipeline test error_handling_test --verbose
../target/debug/oxide_flow pipeline test simple_enhanced_test
../target/debug/oxide_flow pipeline test file_test --dry-run

# Test pipeline execution with sample data
echo '{"test": "data", "number": 42}' | ../target/debug/oxide_flow run simple_enhanced_test
cat test_input.json | ../target/debug/oxide_flow run enhanced_pipeline
../target/debug/oxide_flow run file_test

# Test error handling scenarios
../target/debug/oxide_flow pipeline test invalid_config_test
../target/debug/oxide_flow pipeline test step_reference_test

# Test info and JSON output
../target/debug/oxide_flow pipeline info enhanced_pipeline
../target/debug/oxide_flow pipeline list --json
```

### 4. **Progress Tracking**
After successful implementation:
- Update the plan document to mark the phase as ✅ **COMPLETED**
- Add implementation notes, challenges, and solutions
- Update any changed requirements or scope
- Note any new insights or improvements for future phases

### 5. **Documentation Updates**
- Update relevant documentation files
- Add examples and usage patterns
- Document new CLI commands, functions, or features
- Ensure consistency with existing documentation

## 📝 Implementation Template

Use this structure when implementing a phase:

```
## ✅ Phase X Implementation Complete!

### 🎯 What Was Implemented
- [List key features/components implemented]
- [Note any files created or modified]
- [Highlight important functions or structures added]

### 🧪 Testing Results
- [Summarize testing performed]
- [Note any issues found and resolved]
- [Confirm all functionality works as expected]

### 🔄 Ready for Next Phase
- [State what was accomplished]
- [Note any changes to upcoming phases]
- [Identify next logical implementation steps]

### 📊 Progress Summary
- ✅ Phase X: [Brief description] - COMPLETED
- 🔄 Phase Y: [Brief description] - NEXT
- ⏳ Phase Z: [Brief description] - PENDING
```

## 🎯 Example Usage

**Command:** `implement phase 3 of the pipeline command plan`

**Expected Actions:**
1. Read `docs/current_plan.md`
2. Locate "Phase 3: Pipeline Creation" section
3. Implement template system and pipeline generation
4. Test all new functionality from example_project directory
5. Update plan with ✅ **COMPLETED** status
6. Prepare summary for next phase

**Testing Workflow:**
```bash
# Build and test new features
cargo build
cd example_project

# Test pipeline creation
../target/debug/oxide_flow pipeline add test_pipeline --template basic
../target/debug/oxide_flow pipeline list

# Validate new pipeline
../target/debug/oxide_flow pipeline test test_pipeline --verbose

# Test with existing pipelines
../target/debug/oxide_flow pipeline test enhanced_pipeline
echo '{"name": "test"}' | ../target/debug/oxide_flow run simple_enhanced_test
```

## 🔍 Quality Checklist

Before marking a phase complete, verify:
- [ ] All code compiles without errors: `cargo build`
- [ ] New functionality works as designed
- [ ] Existing functionality remains intact
- [ ] Error handling is appropriate
- [ ] Documentation is updated
- [ ] Plan document reflects current status
- [ ] Next phase dependencies are clear
- [ ] Testing performed from example_project directory
- [ ] All CLI commands tested with sample data

### Integration Tips
- **Reference custom instructions** in prompt files to avoid duplication
- **Make instructions shareable** across team members and projects
- **Version control** all prompt and instruction files for change tracking
- **Test prompts iteratively** using the play button in VS Code editor
- **Use markdown formatting** for better readability and structure

## 📋 Plan Update Format

When updating the plan document, use this format:

```markdown
### Phase X: [Phase Name] ✅ **COMPLETED**
**Status:** Implemented on [DATE]
**Implementation Notes:** [Brief summary of what was built]

1. **[Task 1]** ✅ DONE
   - [Implementation details]
   - [Any changes from original plan]

2. **[Task 2]** ✅ DONE
   - [Implementation details]
   - [Testing notes]

**Next Phase Dependencies:** [Any new requirements or changes affecting future phases]
```

## 🚀 Success Criteria

A phase implementation is successful when:
- All planned functionality is working
- Code quality meets project standards
- Tests pass and edge cases are handled
- Documentation accurately reflects changes
- Plan document is updated with progress
- Team can confidently move to next phase

This structured approach ensures consistent, trackable progress through complex implementation plans while maintaining high quality and clear documentation.
