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

## Configuration

The application uses a YAML configuration file to store your Asana API token and other settings. The default location is platform-specific:

- **macOS/Linux**: `~/.config/asana-tui/config.yaml`
- **Windows**: `%APPDATA%\asana-tui\config.yaml`

### Example Configuration

```yaml
api_token: your_asana_api_token_here
```

## Development

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Build documentation
cargo doc --open
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Benjamin Cofrancesco**

- Email: benjamin@cofrancesco.com
- GitHub: [@bej-cofrancesco](https://github.com/bej-cofrancesco)

## Contributing

Contributions, issues, and feature requests are welcome! Feel free to check the [issues page](https://github.com/bej-cofrancesco/asana-tui/issues).

---

Made with ❤️ using Rust and Ratatui

