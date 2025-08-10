# Rust Code Quality Guidelines

Follow these guidelines to avoid clippy warnings and maintain high code quality:

## Code Style
- **Format strings**: Use direct variable interpolation: `format!("text {var}")` not `format!("text {}", var)`
- **Struct initialization**: Use shorthand when field and variable names match: `MyStruct { field }` not `MyStruct { field: field }`
- **String operations**: Use `strip_prefix()`/`strip_suffix()` instead of manual slicing when checking/removing prefixes/suffixes

## Pattern Matching
- **Error checking**: Use `.is_err()` instead of `if let Err(_) = expr`
- **Single patterns**: Use `if let` instead of single-arm `match` statements
- **Borrowing**: Avoid unnecessary `&` when passing arguments to functions that accept owned values

## Code Organization
- **Derive macros**: Use `#[derive(Default)]` with `#[default]` attribute instead of manual `Default` implementations
- **Closures**: Avoid redundant closures like `|x| SomeType(x)` - use `SomeType` directly
- **Imports**: Remove unused imports to avoid warnings
- **Dead code**: Add `#[allow(dead_code)]` with comments for intentionally unused code (future features)

## Control Flow
- **Nested conditions**: Combine conditions with `&&` instead of nested `if` statements when possible
- **Method parameters**: For functions with >7 parameters, consider using a config struct or add `#[allow(clippy::too_many_arguments)]`

## Recursion
- **Recursive functions**: Add `#[allow(clippy::only_used_in_recursion)]` for parameters only used in recursive calls

## Quality Checks
Always run before committing:
```bash
cargo clippy -- -D warnings  # Zero warnings required
cargo test                    # All tests must pass
```
