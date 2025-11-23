# Rauncher

An Epic Games Store launcher for Linux written in Rust, inspired by [Legendary](https://github.com/derrod/legendary).

## Features

- **GUI-First Design**: Launches directly into a minimal, Epic Games-inspired graphical interface
- **Cross-platform support**: Built for Linux with Rust's performance and safety guarantees
- **Epic Games Store Integration**: Authenticate and access your Epic Games library
- **Game Management**: List, install, launch, and uninstall games
- **Configuration Management**: Persistent configuration and authentication
- **Optional CLI Commands**: Command-line interface available for advanced users and automation

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/kairosci/rauncher.git
cd rauncher

# Build with cargo
cargo build --release

# Install (optional)
cargo install --path .
```

## Usage

### Launch the Application

Simply run the launcher to start the GUI:

```bash
rauncher
```

The GUI provides an Epic Games Store-like experience with:
- **Login Screen**: Authenticate with your Epic Games account
- **Game Library**: Browse all your games with search and filter capabilities
- **Game Cards**: Visual representation of each game with installation status
- **Quick Actions**: Install, launch, or uninstall games with one click
- **Dark Theme**: Modern dark interface inspired by Epic Games Store

### Optional CLI Commands

For advanced users and automation, CLI commands are still available:

### Authentication

Authenticate with Epic Games Store using OAuth device flow:

```bash
rauncher auth
```

This will:
1. Display a verification URL and device code
2. Wait for you to authenticate in your browser
3. Save your authentication token securely

Logout:

```bash
rauncher auth --logout
```

### List Games

List all games in your library:

```bash
rauncher list
```

List only installed games:

```bash
rauncher list --installed
```

### Install a Game

Install a game from your library:

```bash
rauncher install <app_name>
```

### Launch a Game

Launch an installed game:

```bash
rauncher launch <app_name>
```

### Game Information

Show information about a game:

```bash
rauncher info <app_name>
```

### Uninstall a Game

Remove a game:

```bash
rauncher uninstall <app_name>
```

### Update a Game

Check for and install game updates:

```bash
# Check if updates are available
rauncher update <app_name> --check-only

# Update a game
rauncher update <app_name>
```

### Cloud Saves

Manage cloud saves for your games:

```bash
# Download cloud saves
rauncher cloud-save <app_name> --download

# Upload local saves to cloud
rauncher cloud-save <app_name> --upload
```

### Status

Check the launcher status and configuration:

```bash
rauncher status
```

### Options

Enable verbose logging for any command:

```bash
rauncher --verbose <command>
```

## Architecture

The launcher is built with a modular architecture:

- **API Module** (`src/api/`): Epic Games Store API client
- **Auth Module** (`src/auth/`): Authentication and token management
- **Config Module** (`src/config/`): Configuration management
- **Games Module** (`src/games/`): Game installation, launching, and management
- **CLI Module** (`src/cli/`): Command-line interface
- **Error Module** (`src/error.rs`): Error handling

## Configuration

Configuration is stored in:
- **Linux**: `~/.config/rauncher/config.toml`

Default configuration:

```toml
install_dir = "~/.local/share/rauncher/games"
log_level = "info"
```

Authentication tokens are stored securely in:
- **Linux**: `~/.local/share/rauncher/auth.json`

## Development Status

This project is currently in active development. The following features are implemented or in progress:

- [x] Project structure and core modules
- [x] CLI interface
- [x] GUI interface with Epic Games-inspired design
- [x] Configuration management
- [x] Authentication framework
- [x] Game library display (demo mode)
- [x] Game installation workflow (framework in place)
- [x] Game launching (for installed games)
- [x] Game uninstallation
- [x] Epic Games OAuth integration (full implementation)
- [x] Real Epic Games API integration
- [x] Game manifest parsing
- [x] Game download and installation (full implementation)
- [x] Update management
- [x] Cloud saves support

### Implementation Notes

The launcher now includes full implementations for:
- **OAuth Authentication**: Complete device code flow with Epic Games
- **Game Library**: Fetches your owned games from Epic API
- **Manifest Parsing**: Downloads and parses game manifests
- **Game Installation**: Framework with manifest-based installation
- **Update Management**: Check and apply game updates
- **Cloud Saves**: Download and upload save files

**Note on CDN Downloads**: While the manifest parsing and installation framework are complete, the actual CDN chunk download and file reconstruction require Epic Games CDN URLs which vary by game. The current implementation provides the complete structure and can be extended with game-specific CDN configurations.

## Inspiration

This project is inspired by [Legendary](https://github.com/derrod/legendary), an excellent Python-based Epic Games launcher. Rauncher aims to provide similar functionality with the performance and safety benefits of Rust.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

## License

MIT License

## Disclaimer

This project is not affiliated with or endorsed by Epic Games. Use at your own risk.
