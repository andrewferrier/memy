# memy

**memy** is a modern, fast, and simple command-line tool to help you track and recall the files and directories you use most often. Inspired by [fasd](https://github.com/clvv/fasd), memy remembers the paths you interact with, lists them back to you by a combination of frequency and recency ("frecency"), and makes it easy to build a workflow around them using standard Linux/Unix tools.

Unlike tools such as zoxide, memy is less focused on providing a direct "jump" command for navigating directories. Instead, memy is designed to be a flexible backend for tracking your usage, which you can combine with tools like `fzf` and `cd` to jump around directories if you wish. Crucially, memy also supports tracking files you use - not just directories - unlike most other tools in this space (except for `fasd`, which is no longer maintained).

memy is ideal for developers, sysadmins, and anyone who works with many files and directories and wants a smarter way to recall them.

## Quick Start

- Note a file or directory:

  ```sh
  memy note <path>
  ```

- List all remembered paths (in frecency order):

  ```sh
  memy list
  ```

- Change to a directory from your remembered paths using [fzf](https://github.com/junegunn/fzf) as a wrapper:

  ```sh
  cd $(memy list -d | fzf)
  ```

## Installation

### Download Binaries from GitHub

- [Linux (x86_64)](https://github.com/andrewferrier/memy/releases/latest/download/memy-linux-x86_64)
- [macOS (Intel)](https://github.com/andrewferrier/memy/releases/latest/download/memy-darwin-x86_64)
- [macOS (Apple Silicon)](https://github.com/andrewferrier/memy/releases/latest/download/memy-darwin-aarch64)

Download the appropriate binary for your platform, place it somewhere in your `$PATH`, and make it executable if necessary. For example:

```sh
chmod +x memy-<platform-arch>
mv memy-<platform-arch> /usr/local/bin/memy
```

### Install via Cargo (from Source)

If you have [Cargo](https://doc.rust-lang.org/cargo/) installed, you can install memy directly from the latest source:

```sh
cargo install --git https://github.com/andrewferrier/memy
```

#### Don't have Cargo?

Cargo is Rust's package manager and build tool. The easiest way to get Cargo (and Rust) is to use [rustup](https://rustup.rs/), which works on Linux, macOS, and Windows. See the official instructions [here](https://doc.rust-lang.org/cargo/getting-started/installation.html).

## Under the Hood

By default, memy stores its cache data in `$XDG_STATE_HOME/memy/memy.sqlite3` (typically `~/.local/state/memy/memy.sqlite3`). You can override the database location by setting the `MEMY_DB_DIR` environment variable to a directory of your choice.

By default, memy looks for its config file at `$XDG_CONFIG_HOME/memy/memy.toml` (typically `~/.config/memy/memy.toml`). You can override the config location by setting the `MEMY_CONFIG_DIR` environment variable to a directory of your choice.

You can generate a default config file in the default location with:

```sh
memy generate-config
```

## More Information

- For a full list of commands, flags, and options, run:

  ```sh
  memy --help
  ```

- For release notes, see [CHANGELOG.md](CHANGELOG.md)

- Issues and contributions welcome at [https://github.com/andrewferrier/memy](https://github.com/andrewferrier/memy)

## Comparison with Similar Tools

Here's how **memy** compares to other popular directory/file jump and tracking tools:

| Feature                 | memy                                                                                     | zoxide                                                                                   | autojump                                                                             | z                                                                            | fasd                                                                            | fasder                                                                                   |
| ----------------------- | ---------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| **Platforms supported** | ✅ (Linux, macOS)                                                                        | ✅ (Linux, macOS, Windows)                                                               | ✅ (Linux, macOS, Windows)                                                           | ✅ (Linux, macOS, Windows)                                                   | ✅ (Linux, macOS, Windows)                                                      | ✅ (Linux, macOS, Windows)                                                               |
| **Tracks Files**        | ✅                                                                                       | ❌                                                                                       | ❌                                                                                   | ❌                                                                           | ✅                                                                              | ✅                                                                                       |
| **Tracks Directories**  | ✅                                                                                       | ✅                                                                                       | ✅                                                                                   | ✅                                                                           | ✅                                                                              | ✅                                                                                       |
| **Actively Maintained** | ![Last commit](https://img.shields.io/github/last-commit/andrewferrier/memy?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/ajeetdsouza/zoxide?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/wting/autojump?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/rupa/z?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/clvv/fasd?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/ajay-gandhi/fasder?logo=github) |
| **Customizable**        | ✅ (TOML config)                                                                         | ✅ (config file & env vars)                                                              | ✅ (Some)                                                                            | ❌ (Limited)                                                                 | ❌ (Limited)                                                                    | ✅ (config file & env vars)                                                              |
| **Database Format**     | SQLite                                                                                   | SQLite                                                                                   | Text                                                                                 | Text                                                                         | Text                                                                            | Text                                                                                     |
| **Written in**          | Rust                                                                                     | Rust                                                                                     | Python                                                                               | awk/Shell                                                                    | Perl/Shell                                                                      | Go                                                                                       |
