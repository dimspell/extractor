# Integration Tests

This directory contains integration tests for the Dispel GUI application.

## Test Organization

- **`workspace_tests/`**: Tests for workspace management functionality
- **`system_tests/`**: Tests for system-level functionality including command palette

## Running Tests

### Unit Tests (Recommended)

The primary test suite uses Rust's built-in unit test framework with tests embedded in the source files:

```bash
# Run all unit tests
cargo test

# Run tests for a specific package
cargo test -p dispel-gui
```

### Integration Tests

The integration tests in this directory provide higher-level testing but require manual execution:

```bash
# Run a specific integration test file
rustc --test clear_workspace_integration.rs -L target/debug/deps --extern dispel_gui=target/debug/libdispel_gui.rlib && ./clear_workspace_integration
```

## Test Coverage

### Workspace Management
- ✅ Clear workspace functionality
- ✅ Game path preservation
- ✅ Idempotent clearing
- ✅ Tab management

### Command Palette
- ✅ Clear Workspace command existence
- ✅ Command action verification
- ✅ Command organization

## Adding New Tests

For new functionality, prefer adding unit tests directly in the source files using `#[cfg(test)]` modules. This provides:
- Better integration with Rust's test framework
- Automatic test discovery and execution
- Faster compilation and execution

Integration tests in this directory are best suited for:
- End-to-end workflow testing
- Complex interaction testing
- Manual verification scenarios

## Test Conventions

1. **Unit tests**: Test individual functions and modules in isolation
2. **Integration tests**: Test complete workflows and user interactions
3. **Test naming**: Use `test_*` prefix for test functions
4. **Assertions**: Use descriptive assertion messages
5. **Organization**: Group related tests together
