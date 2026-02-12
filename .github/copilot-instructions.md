# Copilot Instructions for memy

## Project Overview

**memy** is a modern, fast CLI tool written in Rust that tracks and recalls frequently/recently used files and directories. Unlike similar tools like zoxide or autojump, memy uniquely **tracks both files and directories**, uses "frecency" scoring (a combination of frequency + recency), and provides a flexible backend that integrates with other tools (fzf, editors, file managers).

### Key Features

- Tracks both files and directories (unique among similar tools except fasd)
- Uses frecency algorithm to rank paths by frequency + recency
- SQLite backend for speed and scalability
- Integration hooks for shells (bash, zsh, fish) and editors (vim, neovim, ranger, lf)
- Can import from fasd, autojump, and zoxide
- Configurable via TOML config file

## Technology Stack

- **Language:** Rust 2024 edition
- **Database:** SQLite (bundled via rusqlite)
- **Build System:** Cargo
- **Core Dependencies:**
  - `clap` - CLI argument parsing with derive macros
  - `rusqlite` - SQLite interface with bundled database
  - `chrono` - Date/time handling for frecency calculations
  - `serde`/`serde_json`/`toml` - Serialization and config parsing
  - `config` - TOML/JSON5 config file loading
  - `tracing`/`log`/`env_logger` - Logging infrastructure
  - `colored` - Terminal output coloring
  - `ignore` - For respecting gitignore patterns in denylist
  - `xdg` - XDG Base Directory specification support

## Build and Validation

### Building

```bash
cargo build              # Debug build
cargo build --release    # Release build with LTO and optimizations
```

The build script (`build.rs`) performs several important tasks:
- Embeds hook files into the binary from the `hooks/` directory
- Generates shell completions (bash, zsh, fish) into `target/completions/`
- Creates man pages from CLI definitions into `target/man/`
- Renders config template with default denylist patterns
- Captures git version information

### Testing

```bash
cargo test               # Run all tests
cargo test --verbose     # Run tests with verbose output
cargo test <test_name>   # Run specific test
```

Tests are organized as:
- Unit tests within source files (using `#[cfg(test)]`)
- Integration tests in `tests/` directory
- Each integration test file tests a specific feature area

### Linting

The project uses extensive clippy linting configured in `Cargo.toml` under `[lints.clippy]`:

```bash
cargo clippy             # Run clippy lints
```

Key clippy configurations:
- Most pedantic, nursery, complexity, and performance lints enabled at warn level
- `unwrap_used` is warned (avoid `.unwrap()` in production code)
- `print_stdout` and `print_stderr` warned (use logging instead)
- `shadow_*` variants warned (avoid variable shadowing)
- `cast_precision_loss` allowed (common in this codebase)
- `multiple_crate_versions` allowed (dependency constraint)

**Important:** The build script (`build.rs`) explicitly allows `unwrap_used` since it's acceptable in build-time code.

### Formatting

```bash
cargo fmt                # Format code
cargo fmt --check        # Check formatting without modifying files
```

### Pre-commit Hooks

The repository uses pre-commit hooks (`.pre-commit-config.yaml`):
- `clippy` - Rust linting
- `gitleaks` - Secret scanning
- `gitlint` - Commit message linting (conventional commits required)
- `editorconfig-checker` - EditorConfig compliance
- TOML/YAML validation

### CI/CD

GitHub Actions workflows in `.github/workflows/`:
- **tests.yml** - Runs on Ubuntu and macOS:
  - `cargo audit` - Security vulnerability checks
  - `cargo build --verbose`
  - `cargo test --verbose`
  - `actionlint` - Validates workflow files
- **codeql.yml** - CodeQL security scanning
- **release-please.yml** - Automated release management
- **release-packages.yml** - Build .deb and .rpm packages
- **check-conventional-commit.yml** - Enforce conventional commits

## Codebase Architecture

### Directory Structure

```
memy/
├── src/                    # Source code
│   ├── main.rs            # Entry point and command routing
│   ├── cli.rs             # CLI argument definitions (clap)
│   ├── db.rs              # SQLite database operations
│   ├── frecency.rs        # Frecency scoring algorithm
│   ├── note.rs            # 'note' command implementation
│   ├── list.rs            # 'list' command implementation
│   ├── stats.rs           # 'stats' command
│   ├── hooks.rs           # Hook management and embedding
│   ├── import.rs          # Import from fasd/autojump/zoxide
│   ├── config.rs          # Configuration loading
│   ├── logging.rs         # Logging setup
│   ├── types.rs           # Common types
│   ├── utils.rs           # Utility functions
│   └── denylist_default.rs # Default denylist patterns
├── hooks/                 # Integration hooks (embedded at build time)
│   ├── bash, zsh, fish    # Shell hooks
│   ├── vim.vim, neovim.lua # Editor hooks
│   └── ranger.rc.conf, lfrc # File manager hooks
├── config/                # Configuration templates
├── tests/                 # Integration tests
│   ├── support.rs         # Test utilities and helpers
│   └── *.rs               # Feature-specific test files
├── benches/               # Performance benchmarks
└── build.rs               # Build script
```

### Key Entry Points

**Commands** (defined in `cli.rs`, routed in `main.rs`):
- `note <PATHS>` - Record file/directory usage
- `list` - Display paths sorted by frecency
  - `-f` - Files only
  - `-d` - Directories only
  - `-n <count>` - Limit results
  - `--newer-than <duration>` - Filter by time
- `stats` - Show usage statistics (plain or JSON format)
- `hook [name]` - Display integration hooks
- `generate-config` - Output default config template
- `completions <shell>` - Generate shell completions
- `import <tool>` - Import from other tools

### Core Modules

**`db.rs`** - Database Layer
- SQLite operations for tracking paths
- Schema management and migrations
- Frecency score calculations and queries
- Path insertion, deletion, and lookup
- Uses bundled SQLite (no external dependency)

**`frecency.rs`** - Scoring Algorithm
- Calculates frecency: combination of frequency and recency
- Configurable recency bias (0.0 to 1.0)
- Time-weighted scoring based on last access times
- Core algorithm for ranking paths

**`config.rs`** - Configuration Management
- Loads TOML config from `$XDG_CONFIG_HOME/memy/memy.toml`
- Environment variable override: `MEMY_CONFIG_DIR`
- Default config values and validation
- Denylist pattern support (glob patterns)

**`hooks.rs`** - Hook Management
- Hooks are embedded in binary at build time via `build.rs`
- Stored in static `HOOKS` map
- Retrieved and printed on demand via `memy hook <name>`

**`import.rs`** - Data Import
- Imports from fasd, autojump, zoxide databases
- One-time import on first run (configurable)
- Handles different database formats (text, SQLite)

**`list.rs`** - List Command
- Queries database with frecency ordering
- Supports filtering by file type, count, time
- Path output with home directory tilde expansion

**`note.rs`** - Note Command
- Records path usage to database
- Path normalization and validation
- Denylist checking
- Symlink resolution (optional)

**`utils.rs`** - Utilities
- Path normalization
- Home directory expansion
- Common helper functions

### Configuration Files

**Config Location:** `$XDG_CONFIG_HOME/memy/memy.toml` (typically `~/.config/memy/memy.toml`)
**Database Location:** `$XDG_STATE_HOME/memy/memy.sqlite3` (typically `~/.local/state/memy/memy.sqlite3`)

Config options:
- `recency_bias` - Weight between frequency and recency (0.0-1.0)
- `denylist` - Glob patterns for paths to exclude
- `import_on_startup` - Auto-import from other tools
- `normalize_paths` - Resolve symlinks

### Database Schema

The SQLite database (`db.rs`) tracks:
- `paths` table: path, type (file/directory), access count, last access time
- Frecency scores computed dynamically based on access patterns
- Indexes for efficient lookup and sorting

### Architectural Principles

From `ARCHITECTURE.md`:
- **No auto_vacuum** - SQLite auto_vacuum disabled as it can worsen performance and database size is not a concern

### Common Patterns

1. **Error Handling:**
   - Avoid `.unwrap()` in production code (clippy warns)
   - Use `?` operator for propagating errors
   - Provide meaningful error messages

2. **Logging:**
   - Use `tracing` macros (`debug!`, `info!`, `warn!`, `error!`)
   - Avoid `print!`/`println!` (clippy warns)
   - Configure via `RUST_LOG` environment variable

3. **Path Handling:**
   - Always normalize paths (resolve `.`, `..`, symlinks as configured)
   - Use tilde (`~`) expansion for home directory in output
   - Check against denylist before recording

4. **Testing:**
   - Integration tests use `assert_cmd` for CLI testing
   - Use `tempfile` for temporary directories/files
   - Test both success and error cases

## Common Issues and Workarounds

### Issue: Build Script Regeneration

**Problem:** Changes to `hooks/` directory don't trigger rebuild.

**Workaround:** The build script includes `println!("cargo:rerun-if-changed=hooks/");` to ensure rebuilds when hooks change. If issues persist, run `cargo clean` and rebuild.

### Issue: Clippy Warnings in Build Script

**Problem:** Build script uses `.unwrap()` which is normally warned by clippy.

**Workaround:** Build script has `#![allow(clippy::unwrap_used, reason = "unwrap() OK inside build")]` at the top since build-time code can safely panic.

### Issue: Git Version in Binary

**Problem:** Git version not updating in releases.

**Workaround:** Build script uses `Command::new("git")` to capture version. Ensure `.git` directory exists and `fetch-depth: 0` in CI (already configured in workflows).

## Development Guidelines

### Code Conventions

- **Use conventional commits** - Required by gitlint and CI
  - Format: `type(scope): description`
  - Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
  - Example: `feat(list): add --newer-than filter`

- **Implement and update unit tests** - Always test new functionality

- **Follow Rust idioms:**
  - Prefer `?` over `.unwrap()` or `.expect()`
  - Use `impl Trait` for return types where appropriate
  - Leverage type system for compile-time guarantees

- **Avoid variable shadowing** - Clippy warns on shadow_* lints

- **Use clippy suggestions** - Fix clippy warnings before committing

### Adding New Features

1. **New Command:**
   - Add command variant to `Cli` enum in `cli.rs`
   - Implement handler in new module (e.g., `src/newcmd.rs`)
   - Add module reference in `main.rs`
   - Add integration test in `tests/newcmd.rs`

2. **New Hook:**
   - Add hook file to `hooks/` directory
   - Build script will automatically embed it
   - Document usage in README.md

3. **Configuration Option:**
   - Add field to `Config` struct in `config.rs`
   - Update `config/template-memy.toml` with documentation
   - Handle in relevant command implementation

### Testing Strategy

- **Unit tests:** Place in same file as implementation using `#[cfg(test)]`
- **Integration tests:** Add to `tests/` directory
- **Use `tests/support.rs`:** Shared test utilities and helpers
- **Test CLI:** Use `assert_cmd::Command` for end-to-end CLI testing
- **Temporary files:** Use `tempfile` crate for test isolation

### Release Process

- Releases managed automatically via `release-please` GitHub Action
- Conventional commits drive version bumping and CHANGELOG generation
- Packages (.deb, .rpm) built automatically on release
- No manual intervention needed for releases
