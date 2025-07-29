# memy

![Tests](https://github.com/andrewferrier/memy/actions/workflows/tests.yml/badge.svg)

**memy** is a modern, fast, and simple command-line tool to help you track and recall the files and directories you use most often. Many similar tools support only directories, but memy supports files too. Inspired by [fasd](https://github.com/clvv/fasd), memy remembers the paths you interact with, lists them back to you by a combination of frequency and recency ("frecency"), and makes it easy to build a workflow around them using standard Linux/Unix tools.

Unlike tools such as [zoxide](https://github.com/ajeetdsouza/zoxide), memy is less focused on providing a direct "jump" command for navigating directories. Instead, memy is designed to be a flexible backend for tracking your usage, which you can combine with tools like `fzf` and `cd` to jump around directories if you wish. Crucially, memy also supports tracking files you use - not just directories - unlike most other tools in this space (except for `fasd`, which is no longer maintained).

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

Many of these more advanced tricks would work well configured as [shell aliases](https://linuxize.com/post/how-to-create-bash-aliases/):

- Change to a directory from your remembered paths using [fzf](https://github.com/junegunn/fzf) as a wrapper:

  ```sh
  cd $(memy list -d | fzf)
  ```

- Change to the most frecent directory containing the string 'download' (case-insensitive):

  ```sh
  cd $(memy list -d | grep -i download | tail -1)
  ```

- Open a recently used file in your editor, selecting it using `fzf` (assuming
  your editor is `vim`.

  ```sh
  vim "$(memy list -f | fzf)"
  ```

- (On Linux) Open a recently used path in your GUI file manager:

  ```sh
  xdg-open "$(memy list -d | fzf)"
  ```

## Installation

### Download Binaries from GitHub

- [Linux (x86_64)](https://github.com/andrewferrier/memy/releases/latest/download/memy-linux-x86_64)
- [Linux (x86_64) - Static Binary using musl](https://github.com/andrewferrier/memy/releases/latest/download/memy-linux-x86_64-musl)

- [Linux (ARM64)](https://github.com/andrewferrier/memy/releases/latest/download/memy-linux-aarch64)
- [Linux (ARM64) - Static Binary using musl](https://github.com/andrewferrier/memy/releases/latest/download/memy-linux-aarch64-musl)

- [MacOS (Apple Silicon)](https://github.com/andrewferrier/memy/releases/latest/download/memy-macos-aarch64)
- [MacOS (Intel)](https://github.com/andrewferrier/memy/releases/latest/download/memy-macos-x86_64)

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

Cargo is Rust's package manager and build tool. The easiest way to get Cargo (and Rust) is to use [rustup](https://rustup.rs/), which works on Linux, macOS, and Windows. See [the official instructions](https://doc.rust-lang.org/cargo/getting-started/installation.html).

## Configuration & Under the Hood

By default, memy looks for its config file at `$XDG_CONFIG_HOME/memy/memy.toml` (typically `~/.config/memy/memy.toml`). You can override the config location by setting the `MEMY_CONFIG_DIR` environment variable to a directory of your choice.

You can generate a default config file in the default location with:

```sh
memy generate-config
```

The config file will be annotated with comments explaining what each option does.

By default, memy stores its cache data in `$XDG_STATE_HOME/memy/memy.sqlite3` (typically `~/.local/state/memy/memy.sqlite3`). You can override the database location by setting the `MEMY_DB_DIR` environment variable to a directory of your choice.

## More Information

- For a full list of commands and flags, run `memy --help`

- For release notes, see [CHANGELOG.md](CHANGELOG.md)

- Issues and contributions welcome at [https://github.com/andrewferrier/memy](https://github.com/andrewferrier/memy)

## Comparison with Similar Tools

Here's how **memy** compares to other popular directory/file jump and tracking tools:

| Feature                 | [memy](https://github.com/andrewferrier/memy)                                            | [zoxide](https://github.com/ajeetdsouza/zoxide)                                          | [autojump](https://github.com/wting/autojump)                                        | [z](https://github.com/rupa/z)                                               | [fasd](https://github.com/clvv/fasd)                                            | [fasder](https://github.com/clarity20/fasder)                                          |
| ----------------------- | ---------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| **Platforms supported** | ✅ (Linux, macOS)                                                                        | ✅ (Linux, macOS, Windows)                                                               | ✅ (Linux, macOS, Windows)                                                           | ✅ (Linux, macOS, Windows)                                                   | ✅ (Linux, macOS, Windows)                                                      | ✅ (Linux, macOS, Windows)                                                               |
| **Tracks Files**        | ✅                                                                                       | ❌                                                                                       | ❌                                                                                   | ❌                                                                           | ✅                                                                              | ✅                                                                                       |
| **Tracks Directories**  | ✅                                                                                       | ✅                                                                                       | ✅                                                                                   | ✅                                                                           | ✅                                                                              | ✅                                                                                       |
| **Actively Maintained** | ![Last commit](https://img.shields.io/github/last-commit/andrewferrier/memy?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/ajeetdsouza/zoxide?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/wting/autojump?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/rupa/z?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/clvv/fasd?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/clarity20/fasder?logo=github) |
| **Customizable**        | ✅ (TOML config)                                                                         | ✅ (config file & env vars)                                                              | ✅ (Some)                                                                            | ❌ (Limited)                                                                 | ❌ (Limited)                                                                    | ✅ (config file & env vars)                                                              |
| **Database Format**     | SQLite                                                                                   | SQLite                                                                                   | Text                                                                                 | Text                                                                         | Text                                                                            | Text                                                                                     |
| **Written in**          | Rust                                                                                     | Rust                                                                                     | Python                                                                               | Shell                                                                    | Shell                                                                      | Go                                                                                       |
