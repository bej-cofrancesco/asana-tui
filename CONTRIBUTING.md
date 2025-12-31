# Contributing to Asana TUI

Thank you for your interest in contributing to Asana TUI! This document provides guidelines and instructions for contributing.

## Development Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/bej-cofrancesco/asana-tui.git
   cd asana-tui
   ```
3. Install dependencies and setup:
   ```bash
   # Install Rust (if not already installed)
   # Visit https://rustup.rs/

   # Install pre-commit hooks
   make install-hooks
   ```

## Code Style

This project follows Rust standard conventions:

- **Formatting**: Use `cargo fmt` to format code. The project uses `rustfmt.toml` for configuration.
- **Linting**: Use `cargo clippy` to check for common issues. See `.clippy.toml` for configuration.
- **Documentation**: Add doc comments for public APIs using `///`.

### Running Checks Locally

Before submitting a PR, ensure all checks pass:

```bash
# Run all CI checks
make ci

# Or individually:
make check      # Check compilation
make test       # Run tests
make fmt-check  # Check formatting
make clippy     # Run linter
```

## Testing

- Add unit tests for new functionality
- Ensure all existing tests pass: `cargo test --all-targets`
- Test your changes manually before submitting

## Submitting Changes

1. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes and commit:
   ```bash
   git add .
   git commit -m "Description of your changes"
   ```
   Pre-commit hooks will run automatically if installed.

3. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

4. Create a Pull Request on GitHub

## Pull Request Guidelines

- **Title**: Clear, descriptive title
- **Description**: Explain what changes you made and why
- **Tests**: Ensure all tests pass
- **CI**: Wait for CI checks to pass
- **Size**: Keep PRs focused and reasonably sized

## Code Review Process

- All PRs require review before merging
- Address review comments promptly
- Keep discussions constructive and respectful

## Project Structure

```
src/
├── app.rs              # Main application logic
├── asana/              # Asana API client
│   ├── client.rs
│   ├── custom_fields.rs
│   ├── error.rs
│   └── mod.rs
├── config/             # Configuration management
├── error.rs             # Error types
├── events/              # Event handling
├── state/               # Application state
├── ui/                  # UI rendering
└── utils/               # Utility functions
```

## Questions?

Feel free to open an issue for questions or discussions about the project.

