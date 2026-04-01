# Pylot

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/FrodeUlr/pylot/rust.yml?branch=main&style=for-the-badge&logo=github)](https://github.com/FrodeUlr/Pylot/actions/workflows/rust.yml)
[![Codecov](https://img.shields.io/codecov/c/github/FrodeUlr/Pylot?style=for-the-badge&logo=codecov&label=CODECOV)](https://codecov.io/github/FrodeUlr/Pylot)

Pylot is a Rust workspace for managing Python virtual environments built with Astral UV.

It provides:

- a CLI for installing, checking, updating, and uninstalling UV
- commands for creating, listing, activating, and deleting Python virtual environments
- a terminal UI for interactive environment management
- shell completion generation for common shells

## Features

- Manage Astral UV from the CLI
- Create virtual environments with a specific Python version
- Install packages directly during environment creation
- Install packages from a requirements file
- Apply default packages from configuration
- Activate or delete environments by name or by interactive selection
- Launch an interactive TUI with `pylot tui`
- Generate completion scripts for `bash`, `zsh`, `fish`, `powershell`, and `elvish`

## Workspace Layout

This repository is split into multiple crates:

- `pylot`: main CLI binary and public library API
- `pylot-core`: pure core domain types and errors
- `pylot-shared`: shared configuration, UV integration, environment management, logging, and utilities
- `pylot-tui`: terminal UI built with `ratatui` and `crossterm`

## Installation

### Prerequisites

- Rust toolchain
- UV is optional at install time because Pylot can install it for you

### Install From Source

Install the CLI from the workspace:

```bash
cargo install --path pylot
```

### Local Development Run

Run the CLI directly from the workspace:

```bash
cargo run -p pylot -- --help
```

Run a specific command:

```bash
cargo run -p pylot -- venv list
```

### Packaged Build

Create a release build and copy the executable plus `settings.toml` into `dist/`:

```bash
make package
```

## Configuration

Pylot reads `settings.toml` from the same directory as the executable.

The repository includes a sample configuration at `pylot/settings.toml`:

```toml
venvs_path = "~/pylot/venvs"
default_pkgs = [
  "neovim",
  "pyvim",
  "pylint",
  "pydantic",
  "jupyter",
  "jupyterthemes",
  "ruff-lsp",
]
```

Settings currently support:

- `venvs_path`: where Pylot stores managed virtual environments
- `default_pkgs`: packages installed when `--default` is used during creation

### Important For Local Development

Because settings are loaded relative to the compiled executable, `cargo run -p pylot` looks for `settings.toml` in `target/debug/`.

You have three practical options during development:

1. run `make debug` to copy `pylot/settings.toml` into `target/debug/`
2. run `make package` and execute the binary from `dist/`
3. manually copy `pylot/settings.toml` next to the compiled executable

If no settings file is found, Pylot falls back to defaults and prints a warning.

## Usage

### UV Management

Check whether UV is installed:

```bash
pylot uv check
```

Install UV:

```bash
pylot uv install
```

Update UV:

```bash
pylot uv update
```

Uninstall UV:

```bash
pylot uv uninstall
```

Aliases:

```bash
pylot u c
pylot u i
pylot u up
pylot u u
```

### Virtual Environment Management

Create a virtual environment with a specific Python version and packages:

```bash
pylot venv create myenv --python-version 3.11 --packages requests numpy
```

Create a virtual environment using default packages from `settings.toml`:

```bash
pylot venv create myenv --default
```

Create a virtual environment and install from a requirements file:

```bash
pylot venv create myenv --requirements requirements.txt
```

List environments:

```bash
pylot venv list
```

Activate an environment by name:

```bash
pylot venv activate myenv
```

Activate interactively:

```bash
pylot venv activate
```

Delete an environment by name:

```bash
pylot venv delete myenv
```

Delete interactively:

```bash
pylot venv delete
```

Short aliases:

```bash
pylot v c myenv -v 3.11 -p requests numpy
pylot v l
pylot v a myenv
pylot v d myenv
```

### TUI

Launch the terminal UI:

```bash
pylot tui
```

The TUI has two tabs: **Environments** and **UV**.

#### Environments tab

| Key | Action |
|-----|--------|
| `n` | Create a new virtual environment (inline form) |
| `d` | Delete the selected environment |
| `Enter` / `a` | Activate the selected environment |
| `i` | Add packages to the selected environment |
| `r` | Remove packages from the selected environment |
| `/` | Search / filter the package list for the selected environment |
| `j` / `k` | Scroll the package list down / up |
| `Tab` / `→` | Switch to the UV tab |
| `q` / `Esc` | Quit |

#### UV tab

| Key | Action |
|-----|--------|
| `i` | Install UV |
| `u` | Update UV |
| `d` | Uninstall UV |
| `Tab` / `→` | Switch to the Environments tab |
| `q` / `Esc` | Quit |

### Shell Completions

Generate completions for supported shells:

```bash
pylot complete bash
pylot complete zsh
pylot complete fish
pylot complete powershell
pylot complete elvish
```

Examples:

```bash
# bash
pylot complete bash > ~/.pylot-completion.bash

# zsh
pylot complete zsh > ~/.pylot-completion.zsh

# fish
pylot complete fish > ~/.config/fish/completions/pylot.fish
```

PowerShell:

```powershell
pylot complete powershell | Out-String | Invoke-Expression
```

## Development

Useful workspace commands:

```bash
make build
make debug
make format
make lint
make test
make coverage
make doc
make package
make clean
```

Direct Cargo equivalents:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test -- --test-threads=1 --no-capture
cargo doc --workspace --no-deps --open
```

### Documentation Notes

`cargo doc --workspace --no-deps` documents the public API across all workspace
crates.  Items inside private modules will not appear unless they are publicly
re-exported or you generate docs with private items enabled.

To include private items in generated documentation:

```bash
cargo doc --workspace --no-deps --document-private-items
```

Running `make doc` runs `cargo test --doc` followed by
`cargo doc --workspace --no-deps`, so the generated HTML in `target/doc/` is
always test-verified.  Append `open` to the Make invocation to open the docs in
a browser automatically:

```bash
make doc open
```

## Testing And CI

The GitHub Actions workflow runs:

- `cargo nextest` for tests on Linux
- `cargo llvm-cov` for coverage on Linux and Windows
- Codecov uploads for coverage and JUnit test results
- release packaging into workflow artifacts

The Linux nextest profile writes `junit.xml`, which is used for Codecov test result uploads.

## License

This project is licensed under the terms of the [LICENSE](LICENSE) file.
