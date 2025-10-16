# Contributing to WisePulse

Thank you for your interest in contributing to WisePulse! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- **Rust** (latest stable): Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Docker and Docker Compose**: For running SILO preprocessing and API
- **Git**: For version control

### Getting Started

1. **Clone the repository**
   ```bash
   git clone https://github.com/cbg-ethz/WisePulse.git
   cd WisePulse
   ```

2. **Build the Rust utilities**
   ```bash
   make build
   # or
   cargo build --release
   ```

3. **Run the test suite**
   ```bash
   cargo test --workspace
   ```

## Code Quality Standards

We maintain high code quality standards through automated checks:

### Formatting

All Rust code must be formatted with `rustfmt`:

```bash
# Check formatting
cargo fmt --check

# Auto-format code
cargo fmt
```

### Linting

Code must pass `clippy` checks without warnings:

```bash
# Run clippy
cargo clippy --workspace --all-targets -- -D warnings

# Auto-fix some issues
cargo clippy --fix --workspace --allow-dirty
```

### Testing

While we don't have extensive unit tests yet, all code should:
- Compile without warnings
- Pass existing integration tests
- Be manually tested for correctness

```bash
# Run all tests
cargo test --workspace

# Run end-to-end test
make build
cp test_data/* silo_input
make all
```

## Continuous Integration

All pull requests are automatically checked for:
- ✅ Code formatting (`rustfmt`)
- ✅ Linting (`clippy`)
- ✅ Successful build
- ✅ Test passage
- ✅ End-to-end integration test

Make sure your code passes these checks before submitting a PR.

## Pull Request Process

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write clean, well-documented code
   - Follow existing code style and patterns
   - Add comments for complex logic

3. **Test your changes**
   ```bash
   cargo fmt
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

4. **Commit your changes**
   - Use clear, descriptive commit messages
   - Reference any related issues

5. **Push and create a Pull Request**
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Address review feedback**
   - Be responsive to comments
   - Make requested changes promptly

## Project Structure

```
WisePulse/
├── add_offset/              # Rust: Add offset field to sequences
├── check_new_data/          # Rust: Check for new data availability
├── fetch_silo_data/         # Rust: Fetch data from LAPIS API
├── merge_sorted_chunks/     # Rust: Merge sorted data chunks
├── split_into_sorted_chunks/# Rust: Split and sort data chunks
├── ansible/                 # Ansible playbooks and roles
├── test_data/              # Test data for CI/CD
├── Makefile                # Build and orchestration
└── .github/workflows/      # CI/CD pipelines
```

## Rust Workspace

This is a Cargo workspace containing multiple binary crates. Each utility is self-contained but shares common dependencies.

### Adding Dependencies

When adding new dependencies:
- Prefer stable, well-maintained crates
- Use specific version constraints
- Document why the dependency is needed

### Code Organization

- Keep binaries focused on single responsibilities
- Extract shared logic into functions
- Use descriptive variable and function names
- Add documentation comments for public APIs

## Common Tasks

### Adding a New Utility

1. Create a new directory in the workspace
2. Add it to `Cargo.toml` members list
3. Implement the utility
4. Update the Makefile if needed
5. Document usage in README.md

### Updating Dependencies

```bash
# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update

# Test after updating
cargo test --workspace
```

## Getting Help

- **Issues**: Open an issue for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Check the main [README.md](README.md) and [ansible/README.md](ansible/README.md)

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the code, not the person
- Help others learn and grow

## License

By contributing to WisePulse, you agree that your contributions will be licensed under the MIT License.
