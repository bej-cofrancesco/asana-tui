# Asana TUI

A beautiful terminal user interface for managing your Asana tasks and projects.

## Features

- 🎨 **Beautiful Terminal UI** - Built with Ratatui for a modern terminal experience
- 📋 **Task Management** - View, create, edit, and manage your Asana tasks
- 📊 **Kanban View** - Visualize your tasks in a kanban board layout
- 🔍 **Task Details** - View detailed information about tasks
- ⚡ **Fast & Efficient** - Lightweight and responsive terminal application
- 🔐 **Secure** - Uses Asana API for secure authentication

## Installation

### Prerequisites

- Rust (latest stable version)
- Asana API token

### Build from Source

```bash
# Clone the repository
git clone https://github.com/bej-cofrancesco/asana-tui.git
cd asana-tui

# Build the project
cargo build --release

# Run the application
cargo run --release
```

## Usage

```bash
# Run with default configuration
asana-tui

# Run with custom config file
asana-tui -c /path/to/config.yaml
```

## Development

### Prerequisites

- Rust (latest stable version)
- `rustfmt` and `clippy` (usually included with Rust)

### Setup

```bash
# Clone the repository
git clone https://github.com/bej-cofrancesco/asana-tui.git
cd asana-tui

### Common Tasks

```bash
# Check code compiles
cargo check --all-targets

# Run tests
cargo test --all-targets

# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Build release binary
cargo build --release
```

### CI/CD

This project uses GitHub Actions for continuous integration. The CI pipeline:

- ✅ Checks code compiles (`cargo check`)
- ✅ Runs all tests (`cargo test`)
- ✅ Checks code formatting (`cargo fmt --check`)
- ✅ Runs clippy linter (`cargo clippy`)
- ✅ Builds release binaries

See `.github/workflows/ci.yml` for details.

## Configuration

The application uses a YAML configuration file to store your Asana API token and other settings. The default location is platform-specific:

- **macOS/Linux**: `~/.config/asana-tui/config.yaml`
- **Windows**: `%APPDATA%\asana-tui\config.yaml`

### Example Configuration

```yaml
api_token: your_asana_api_token_here
```

### Documentation

- **Rust Docs**: Run `cargo doc --open` to view generated Rust documentation
- **Contributing**: See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Benjamin Cofrancesco**

- Email: ben@enxystudio.com
- GitHub: [@bej-cofrancesco](https://github.com/bej-cofrancesco)

## Benchmarks

Performance benchmarks are available for critical code paths:

```bash
# Run all benchmarks
cargo bench
```

Benchmarks are located in the `benches/` directory and cover:
- Custom field validation and building
- Text processing utilities

## Contributing

Contributions, issues, and feature requests are welcome! Feel free to check the [issues page](https://github.com/bej-cofrancesco/asana-tui/issues).

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed contribution guidelines.

---

Made with ❤️ using Rust and Ratatui

