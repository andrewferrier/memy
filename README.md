# memy

![Tests](https://github.com/andrewferrier/memy/actions/workflows/tests.yml/badge.svg)

<p align="center">
<img src="./logo.svg" />
</p>
<p align="center">
<strong><a href="https://youtu.be/YKfYz_56nkg">YouTube Demo</a></strong><br/>(<a href="https://www.youtube.com/playlist?list=PLFbUBvOvJa4S7aoNOY6lf155NCj_nfrQm">other videos on memy</a>)
<br/>
</p>

**memy** is a modern, fast command-line tool to track and recall the files and directories you use most often. It hooks into a variety of common tools and remembers every path you interact with, ranking them by a combination of frequency and recency ("frecency"). Unlike most similar tools, memy tracks both files and directories. It's written in Rust with a SQLite database for speed and scalability, and is ideal for CLI power users.

memy can be used in three interchangeable ways:

- **Zoxide-compatible directory jumping** - memy provides a `z` command that works as a drop-in replacement for [zoxide](https://github.com/ajeetdsouza/zoxide), letting you jump to frecent directories with partial string matching.
- **Interactive selection with tools like [`fzf`](https://github.com/junegunn/fzf)** — memy integrates with fuzzy finders and other selectors so you can interactively pick and open files or directories. Out of the box, the memy shell hooks provide ready-made functions (`memy-cd`, `memy-open`, `memy-go`) for common workflows.
- **Structured output for custom workflows** - memy can output your tracked paths as plain text, CSV, or JSON to script your own workflows or build integrations from scratch.

memy was originally inspired by [fasd](https://github.com/whjvenyl/fasd), but this was been unmaintained for some time, and so I decided to start memy afresh with some modern implementation choices. memy has been tested on Linux and MacOS (limited). It has not been tested on Windows, any testing or feedback would be appreciated. `memy` is partially created using AI assistance - all code changes are overseen by a human maintainer!

## Quick Start

- Note a file or directory:

  ```sh
  memy note <path>
  ```

  You are free to note a path whenever you wish, although typically this is done by the supplied hooks (see more information below).

- List all remembered paths (in frecency order):

  ```sh
  memy list
  ```

  (`ls` is an alias for `list`, so `memy ls` works too.)

- Open a recently used file in your editor, selecting it using `fzf` or other selector (assuming your editor is `vim`).

  ```sh
  memy list -f -s | xargs vim
  ```

- Change to a directory from your remembered paths using [fzf](https://github.com/junegunn/fzf) as a wrapper:

  ```sh
  cd $(memy list -d -s)
  # or use the `memy-cd` convenience command if the memy hook is installed for your shell (see below)
  ```

- Change to the most frecent directory containing the string 'download' (case-insensitive):

  ```sh
  cd $(memy list -d -s --output-filter-command 'grep -i download | head -1')
  ```

- Open a recently used file with the platform default application, selecting it using `fzf` or other selector:

  ```sh
  memy list -f -s | xargs xdg-open   # Linux
  memy list -f -s | xargs open       # macOS
  # or use the `memy-open` convenience command if the memy hook is installed for your shell (see below)
  ```

- Select from all remembered paths and `cd` to it if it's a directory, or open it with the default application if it's a file:

  ```sh
  # use the `memy-go` convenience command if the memy hook is installed for your shell (see below)
  memy-go
  ```

Many of these more advanced tricks would work well configured as [shell aliases](https://linuxize.com/post/how-to-create-bash-aliases/).

`memy` will import your database from [fasd](https://github.com/whjvenyl/fasd), [autojump](https://github.com/wting/autojump), [jumper](https://github.com/homerours/jumper) and/or [zoxide](https://github.com/ajeetdsouza/zoxide), if there is one, on first run (this behaviour can be disabled in the configuration file).

## Zoxide-Compatible `z` Command

If you have the memy shell hook installed (see below), memy also provides a `z` command that works as a drop-in replacement for [zoxide](https://github.com/ajeetdsouza/zoxide)'s `z`. It lets you jump to your most frecently-used directories with just a few keystrokes, using the same keyword-matching algorithm as zoxide:

```sh
z bar        # jump to the most frecent directory whose last component contains 'bar'
z foo bar    # jump to the most frecent directory matching 'foo' and then 'bar' (in order)
```

`z` also handles common path shortcuts directly without consulting the database:

```sh
z ~/projects   # jump straight to ~/projects (or any tilde/absolute path that exists)
z ..           # go up one directory
```

If you already have zoxide installed, memy's `z` will not override it — the function is only defined if `z` does not already exist in your shell.

`zi` is also provided as a zoxide-compatible interactive variant of `z`. It filters directories using the same keyword-matching algorithm as `z`, then lets you pick interactively via `fzf` or a similar output filter:

```sh
zi          # interactively pick from all noted directories
zi bar      # filter to directories matching 'bar', then pick interactively
zi foo bar  # filter to directories matching 'foo' then 'bar', then pick interactively
```

## Noting files automatically using hooks

Hooks in memy are scripts or other configuration files provided with memy that can be embedded into other tools' configurations. These hooks allow you to automatically note files as they are used, opened, or interacted with, integrating memy seamlessly into your workflow.

For example, you might use a hook to automatically note files opened in your text editor or accessed via the command line, or directories you change to in your shell. Hooks are designed to be a starting point only and can be customized to suit your specific needs and preferences. Over time, we plan to grow the list of hooks available. Any contributions to the predefined hooks available would be very welcome as issues or pull requests [on this repository](https://github.com/andrewferrier/memy).

### Using Hooks

To see the list of current hooks provided by memy, type `memy hook`.

To see the contents of a hook, type `memy hook <hookname>`. In future, we'll provide an easier way to [automatically install some hooks](https://github.com/andrewferrier/memy/issues/53). For now, the provided hooks can be installed like this (please be careful to make sure you backup any configuration files etc. before running these commands to avoid mishaps):

| Tool   | How to Install                                                                                                        |
| ------ | --------------------------------------------------------------------------------------------------------------------- |
| bash   | Run `echo 'source <(memy hook bash)' >> ~/.bashrc`                                                                    |
| fish   | Run `memy hook fish.fish >> ~/.config/fish/config.fish`                                                               |
| lfrc   | Run `memy hook lfrc >> ~/.config/lf/lfrc`                                                                             |
| neovim | Run `memy hook neovim.lua > ~/.config/nvim/plugin/memy.lua`                                                           |
| ranger | Run `memy hook ranger.rc.conf >> ~/.config/ranger/rc.conf`                                                            |
| vim    | Run `memy hook vim.vim > ~/.vim/plugin/memy.vim`                                                                      |
| zsh    | Run `echo 'eval $(memy hook zsh)' >> ~/.zshrc` or add `https://github.com/andrewferrier/memy` to your zsh plugin list |

### Shell Convenience Functions

When the bash, zsh, or fish hook is installed, the following shell functions are available:

- **`memy-cd`** — select a directory from your remembered paths using your configured selector (e.g. fzf), then `cd` to it.
- **`memy-open`** — select a file from your remembered paths using your configured selector, then open it with the platform default application (`xdg-open` on Linux, `open` on macOS).
- **`memy-go`** — select from all remembered paths (files and directories); `cd` if the selection is a directory, or open with the default application if it is a file.

## Installation

### Automated Install using Shell Script

Use a single shell script to install:

```
curl -sSL https://raw.githubusercontent.com/andrewferrier/memy/main/install.sh | sh
```

### Homebrew (Linux or Mac)

Install homebrew as per [the instructions](https://brew.sh/). Then run:

```sh
brew tap ferriera/memy https://github.com/andrewferrier/memy.git
brew install ferriera/memy/memy
```

### Download .deb-based package for Debian / Ubuntu

- [x86_64/amd64](https://github.com/andrewferrier/memy/releases/latest/download/memy_latest_amd64.deb)

- [ARM64](https://github.com/andrewferrier/memy/releases/latest/download/memy_latest_arm64.deb)

[Install using dpkg or apt](https://unix.stackexchange.com/a/159114/18985). Currently, Debian packages are not in a signed repository.

### Download .rpm-based package for RHEL / Fedora / CentOS / OpenSUSE / SLES

- [x86_64/amd64](https://github.com/andrewferrier/memy/releases/latest/download/memy_latest_amd64.rpm)

- [ARM64](https://github.com/andrewferrier/memy/releases/latest/download/memy_latest_arm64.rpm)

[How to install RPMs](https://phoenixnap.com/kb/how-to-install-rpm-file-centos-linux).

### Download Binaries for Linux or MacOS from GitHub

- [Linux (x86_64)](https://github.com/andrewferrier/memy/releases/latest/download/memy-linux-x86_64)

- [Linux (x86_64) - Static Binary using musl](https://github.com/andrewferrier/memy/releases/latest/download/memy-linux-x86_64-musl)

- [Linux (ARM64)](https://github.com/andrewferrier/memy/releases/latest/download/memy-linux-aarch64)

- [Linux (ARM64) - Static Binary using musl](https://github.com/andrewferrier/memy/releases/latest/download/memy-linux-aarch64-musl)

The binaries for MacOS are not currently signed, and so [you will have to work around this](https://www.macworld.com/article/672947/how-to-open-a-mac-app-from-an-unidentified-developer.html).

- [MacOS (Apple Silicon)](https://github.com/andrewferrier/memy/releases/latest/download/memy-macos-aarch64)
- [MacOS (Intel)](https://github.com/andrewferrier/memy/releases/latest/download/memy-macos-x86_64)

Download the appropriate binary for your platform, place it somewhere in your `$PATH`, and make it executable if necessary. For example:

```sh
chmod +x memy-<platform-arch>
mv memy-<platform-arch> /usr/local/bin/memy
```

### Install via Cargo (from Source)

If you have [Cargo](https://doc.rust-lang.org/cargo/) installed, you can install memy directly from the very latest source (main branch). This version may have more recent changes than the packaged versions linked above and so may be more unstable.

```sh
cargo install --git https://github.com/andrewferrier/memy
```

Don't have Cargo? It's Rust's package manager and build tool. The easiest way to get Cargo (and Rust) is to use [rustup](https://rustup.rs/), which works on Linux, macOS, and Windows. See [the official instructions](https://doc.rust-lang.org/cargo/getting-started/installation.html).

### Shell Completions

memy can generate tab-completion scripts for your shell:

**Bash:**

```sh
memy completions bash > "${HOME}/.local/share/bash-completion/completions/memy"
```

**Zsh** (add `~/.zfunc` to `$fpath` in `~/.zshrc` if not already present):

```sh
mkdir -p "${HOME}/.zfunc"
memy completions zsh > "${HOME}/.zfunc/_memy"
```

**Fish:**

```sh
memy completions fish > "$HOME/.config/fish/completions/memy.fish"
```

## Configuration & Under the Hood

By default, memy looks for its config file at `$XDG_CONFIG_HOME/memy/memy.toml` (typically `~/.config/memy/memy.toml`). You can override the config location by setting the `MEMY_CONFIG_DIR` environment variable to a directory of your choice.

If you don't already have a config file, you can create a default/template one in the default location, annotated with comments explaining what each option does. Run this command to create it (being careful not to overwrite one that already exists):

```sh
memy generate-config > ~/.config/memy/memy.toml
```

By default, memy stores its database in `$XDG_STATE_HOME/memy/memy.sqlite3` (typically `~/.local/state/memy/memy.sqlite3`). You can override the database location by setting the `MEMY_DB_DIR` environment variable to a directory of your choice.

## Frecency Scoring & Directory Matching

### How Frecency Is Calculated

Every path (file or directory) in memy's database has a **noted count** (how many times it has been noted) and a **last noted timestamp**. When paths are listed with `memy list`, they are sorted by increasing frecency score - a number between `0` and `1` that blends frequency and recency:

```math
\text{frecency} = (1 - \lambda) \cdot \frac{c}{c_{\max}} + \lambda \cdot \left(1 - \frac{t}{t_{\max}}\right)
```

| Symbol       | Meaning                                         |
| ------------ | ----------------------------------------------- |
| $\\lambda$   | `recency_bias` config parameter (default `0.5`) |
| $c$          | Noted count for this entry                      |
| $c\_{\\max}$ | Highest noted count across all entries          |
| $t$          | Age of this entry's last note, in hours         |
| $t\_{\\max}$ | Age of the oldest entry's last note, in hours   |

The first term is the **frequency score** — how often this path has been noted,
normalised against the most-noted path. The second term is the **recency
score** — how recently it was last noted, normalised so that the most recently
noted entry scores close to 1 and the oldest entry scores 0.

The `recency_bias` parameter (λ) controls the weighting:

- `recency_bias = 0.0` — pure frequency; recency is ignored entirely.
- `recency_bias = 1.0` — pure recency; frequency is ignored entirely.
- `recency_bias = 0.5` *(default)* — equal weight to both.

### How `z` Keyword Matching Works

The `z`/`zi` commands match directories using the same algorithm as
[zoxide](https://github.com/ajeetdsouza/zoxide). For full details see the
[zoxide algorithm documentation](https://github.com/ajeetdsouza/zoxide/wiki/Algorithm). The `z` command always returns the most frecent directory that matches, and the `zi` command presents directories in a list by frecency after applying the `z` algorithm. Note that memy's frecency algorithm is slightly different from zoxide's.

## More Information

- For a full list of commands and flags, run `memy --help`. Depending on your memy installation method, you may also be able to bring up a manpage: `man memy`.

- For release notes, see [CHANGELOG.md](CHANGELOG.md)

- Issues and contributions welcome at [https://github.com/andrewferrier/memy](https://github.com/andrewferrier/memy)

## Comparison with Similar Tools

Here's how **memy** compares to other popular directory/file jump and tracking tools:

| Feature                                            | [memy](https://github.com/andrewferrier/memy)                                            | [zoxide](https://github.com/ajeetdsouza/zoxide)                                          | [jumper](https://github.com/homerours/jumper)                                          | [autojump](https://github.com/wting/autojump)                                        | [z](https://github.com/rupa/z)                                               | [fasd](https://github.com/whjvenyl/fasd)                                            | [fasder](https://github.com/clarity20/fasder)                                          |
| -------------------------------------------------- | ---------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| **Platforms supported**                            | ✅ (Linux, macOS)                                                                        | ✅ (Linux, macOS, Windows)                                                               | ✅ (Linux, macOS)                                                                      | ✅ (Linux, macOS, Windows)                                                           | ✅ (Linux, macOS, Windows)                                                   | ✅ (Linux, macOS, Windows)                                                          | ✅ (Linux, macOS, Windows)                                                             |
| **Tracks Files**                                   | ✅                                                                                       | ❌                                                                                       | ✅                                                                                     | ❌                                                                                   | ❌                                                                           | ✅                                                                                  | ✅                                                                                     |
| **Tracks Directories**                             | ✅                                                                                       | ✅                                                                                       | ✅                                                                                     | ✅                                                                                   | ✅                                                                           | ✅                                                                                  | ✅                                                                                     |
| **Supports `z`/`zi` commands (zoxide-compatible)** | ✅                                                                                       | ✅ (the originator)                                                                      | ✅                                                                                     | ❌ (uses `j` command)                                                                | ⚠️ (has `z` but predates/differs from zoxide)                                | ⚠️ (has `z` alias but different algorithm)                                          | ⚠️ (has `z` alias but different algorithm)                                             |
| **Actively Maintained**                            | ![Last commit](https://img.shields.io/github/last-commit/andrewferrier/memy?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/ajeetdsouza/zoxide?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/homerours/jumper?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/wting/autojump?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/rupa/z?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/whjvenyl/fasd?logo=github) | ![Last commit](https://img.shields.io/github/last-commit/clarity20/fasder?logo=github) |
| **GitHub Stars**                                   | ![Stars](https://img.shields.io/github/stars/andrewferrier/memy?style=flat-square)       | ![Stars](https://img.shields.io/github/stars/ajeetdsouza/zoxide?style=flat-square)       | ![Stars](https://img.shields.io/github/stars/homerours/jumper?style=flat-square)       | ![Stars](https://img.shields.io/github/stars/wting/autojump?style=flat-square)       | ![Stars](https://img.shields.io/github/stars/rupa/z?style=flat-square)       | ![Stars](https://img.shields.io/github/stars/whjvenyl/fasd?style=flat-square)       | ![Stars](https://img.shields.io/github/stars/clarity20/fasder?style=flat-square)       |
| **Configuration method**                           | TOML config file                                                                         | env vars                                                                                 | env vars                                                                               | env vars                                                                             | env vars                                                                     | env vars                                                                            | env vars                                                                               |
| **Denylist / path exclusions**                     | ✅ (gitignore-style patterns)                                                            | ✅ (glob env var)                                                                        | ✅ (filter file `~/.jfilters`)                                                         | ❌                                                                                   | ✅ (directory array env var)                                                 | ❌                                                                                  | ❌                                                                                     |
| **Configurable recency bias**                      | ✅                                                                                       | ❌                                                                                       | ✅ (via `--beta` flag)                                                                 | ❌                                                                                   | ❌                                                                           | ❌                                                                                  | ❌                                                                                     |
| **Structured output (JSON/CSV)**                   | ✅                                                                                       | ❌                                                                                       | ❌                                                                                     | ❌                                                                                   | ❌                                                                           | ❌                                                                                  | ❌                                                                                     |
| **Import from other tools**                        | ✅ (automatic on first use)                                                              | ✅ (manual `zoxide import` command)                                                      | ⚠️ (manual `cp ~/.z ~/.jfolders`)                                                      | ❌                                                                                   | ❌                                                                           | ❌                                                                                  | ❌                                                                                     |
| **Editor integration hooks**                       | ✅ (Vim, Neovim)                                                                         | ✅ (third-party integrations: Vim, Neovim, Emacs)                                        | ✅ (Vim, Neovim, VSCode)                                                               | ❌                                                                                   | ❌                                                                           | ❌                                                                                  | ❌                                                                                     |
| **File manager hooks**                             | ✅ (lf, ranger)                                                                          | ✅ (third-party integrations: yazi, lf, ranger, and more)                                | ❌                                                                                     | ❌                                                                                   | ❌                                                                           | ❌                                                                                  | ❌                                                                                     |
| **Auto-cleanup of stale entries**                  | ✅ (configurable: N days, or never)                                                      | ✅ (score threshold)                                                                     | ✅ (configurable frequency via env var)                                                | ✅                                                                                   | ✅ (score aging)                                                             | ❌                                                                                  | ❌                                                                                     |
| **Database Format**                                | SQLite                                                                                   | SQLite                                                                                   | Text                                                                                   | Text                                                                                 | Text                                                                         | Text                                                                                | Text                                                                                   |
| **Uses XDG-compliant storage**                     | ✅                                                                                       | ✅                                                                                       | ❌                                                                                     | ✅ (Linux; macOS uses `~/Library`)                                                   | ❌                                                                           | ❌                                                                                  | ❌                                                                                     |
| **Written in**                                     | Rust                                                                                     | Rust                                                                                     | C                                                                                      | Python                                                                               | Shell                                                                        | Shell                                                                               | Go                                                                                     |
