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
- **Build System:** Cargo

For detailed dependency information, refer to `Cargo.toml`.

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

```bash
cargo clippy             # Run clippy lints
```

The project uses extensive clippy linting configured in `Cargo.toml` under `[lints.clippy]`. Refer to `Cargo.toml` for the complete configuration. **Ensure all code produced conforms to the clippy configuration.**

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
├── hooks/                  # Integration hooks (embedded at build time)
├── config/                 # Configuration templates
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
└── build.rs                # Build script
```

### Architectural Principles

Refer to `ARCHITECTURE.md` for architectural principles and design decisions.

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
