# Contributing to mindmap-cli

Thank you for your interest in contributing to mindmap-cli! This document provides guidelines and information for contributors.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Reporting Issues](#reporting-issues)
- [License](#license)

## Getting Started

### Prerequisites

- **Rust**: Install the latest stable version from [rustup.rs](https://rustup.rs/)
- **Git**: Version control system
- **Optional**: [mise](https://mise.jdx.dev/) for environment management

### Quick Setup

1. **Fork and Clone** the repository:
   ```bash
   git clone https://github.com/your-username/mindmap-cli.git
   cd mindmap-cli
   ```

2. **Set up development environment**:
   ```bash
   # If using mise (recommended)
   mise install
   mise use

   # Or install Rust manually
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Verify installation**:
   ```bash
   cargo --version
   rustc --version
   cargo test
   ```

## Development Setup

### Building

```bash
# Build in debug mode
cargo build

# Build optimized release
cargo build --release

# Run the application
cargo run -- --help
```

### Development Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Generate documentation
cargo doc --open
```

## Development Workflow

### 1. Choose an Issue

- Check [open issues](https://github.com/roobie/mindmap-cli/issues) on GitHub
- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to indicate you're working on it

### 2. Create a Branch

```bash
# Create and switch to a feature branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 3. Make Changes

- Write clear, focused commits
- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed

### 4. Test Your Changes

```bash
# Run the full test suite
cargo test

# Run specific tests related to your changes
cargo test --lib

# Manual testing with sample data
cargo run -- --file test.mindmap show 1
```

### 5. Commit Your Changes

```bash
# Stage your changes
git add .

# Commit with a clear message
git commit -m "feat: add awesome new feature

- Add detailed description of changes
- Explain why the change was made
- Reference any related issues"
```

Use conventional commit format:
- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation
- `refactor:` for code restructuring
- `test:` for test additions

## Code Style

### Rust Style Guidelines

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/style-guide/)
- Use `cargo fmt` to format your code
- Use `cargo clippy` for linting
- Write idiomatic Rust code

### Code Structure

- **Error Handling**: Use `Result<T, E>` and `anyhow` for errors
- **Documentation**: Add doc comments to public functions
- **Naming**: Use descriptive names following Rust conventions
- **Modularity**: Keep functions focused and modules well-organized

### Examples

```rust
// Good: Clear naming and documentation
/// Validates that a node ID exists in the mindmap
pub fn validate_node_exists(mm: &Mindmap, id: u32) -> Result<()> {
    if !mm.by_id.contains_key(&id) {
        return Err(anyhow!("Node [{}] not found", id));
    }
    Ok(())
}

// Avoid: Unclear naming and lack of documentation
fn check(id: u32, m: &Mindmap) -> Result<()> {
    if !m.by_id.contains_key(&id) {
        Err(anyhow!("not found"))
    } else {
        Ok(())
    }
}
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with detailed output
cargo test -- --nocapture

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration

# Run tests matching a pattern
cargo test validate
```

### Writing Tests

- **Unit Tests**: Test individual functions in `src/lib.rs`
- **Integration Tests**: Test command-line interface in `tests/`
- **Test Coverage**: Aim for good coverage of edge cases

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_validation() {
        let temp = assert_fs::TempDir::new().unwrap();
        let file = temp.child("test.md");
        file.write_str("[1] **AE: Test** - body").unwrap();

        let mm = Mindmap::load(file.path().to_path_buf()).unwrap();

        // Test valid node
        assert!(validate_node_exists(&mm, 1).is_ok());

        // Test invalid node
        assert!(validate_node_exists(&mm, 999).is_err());

        temp.close().unwrap();
    }
}
```

### Test Guidelines

- Use descriptive test names
- Test both success and failure cases
- Use `assert_fs` for temporary file testing
- Clean up test resources properly

## Pull Request Process

### Before Submitting

1. **Update Documentation**: Ensure README and docs reflect your changes
2. **Run Full Test Suite**: `cargo test` should pass
3. **Format and Lint**: `cargo fmt && cargo clippy`
4. **Test Manual Usage**: Verify your changes work in practice

### Creating a Pull Request

1. **Push your branch** to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create PR on GitHub**:
   - Base branch: `main` or `development`
   - Clear title and description
   - Reference related issues
   - Add screenshots/demo for UI changes

3. **PR Description Template**:
   ```markdown
   ## Description
   Brief description of the changes

   ## Changes Made
   - Bullet point list of changes
   - Include rationale for decisions

   ## Testing
   - How you tested the changes
   - Edge cases considered

   ## Related Issues
   Closes #123, Fixes #456

   ## Screenshots (if applicable)
   ```

### Review Process

- **Automated Checks**: CI will run tests and linting
- **Code Review**: Maintainers will review your code
- **Feedback**: Address review comments
- **Approval**: PR will be merged when approved

## Reporting Issues

### Bug Reports

When reporting bugs, please include:

- **Steps to reproduce** the issue
- **Expected behavior** vs actual behavior
- **Environment**: OS, Rust version, mindmap-cli version
- **Sample input** that triggers the bug
- **Error messages** or stack traces

### Feature Requests

For feature requests, please:

- **Describe the problem** you're trying to solve
- **Explain your proposed solution**
- **Consider alternatives** you've thought about
- **Provide examples** of how it would be used

## License

By contributing to mindmap-cli, you agree that your contributions will be licensed under the same license as the project (see LICENSE file).

Thank you for contributing to mindmap-cli! 🎉