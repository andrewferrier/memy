# Screencast Script: memy

- Open a window into the memy repo, `cd demo/` and run `make`
- Open browser to:
    <https://github.com/andrewferrier/memy>
    <https://github.com/whjvenyl/fasd>
    <https://github.com/ajeetdsouza/zoxide>
    <https://github.com/junegunn/fzf>

## Introduction

- Introduce memy: a fast, modern CLI tool for tracking and recalling frequently used files and directories using frecency algorithms, which combine frequency and recency
- Designed for those who are comfortable working with the command line and hacking on their system
- Many alternatives such as zoxide which only track directories
- Also fasd which tracks both but is based entirely in shell script and has a number of limitations that memy overcomes with a modern Rust implementation and SQLite database
- memy will import database from those tools on first run
- It's a 'backend', building frontends is up to you

## Basic Usage

- memy note a file
- memy note a dir
- memy list
- memy list -f
- memy list -d
- memy note a file x 3
- memy list
- memy list --format=json
- memy --help, memy stats
- delete file
- memy list
- reinstate file
- memy list and explain
- memy list | fzf
- memy note a few more files
- Scroll to README and find hook for editing, run it
- What I mean when I say memy doesn't provide much frontend
- Wrap it as a shell alias

- Scroll to README and show hook install for bash, run that
- Run `bash --login`
- Now navigate around and show directories / files building up
- Now use memy-cd

## Conclusion

- memy is a database for tracking frequently used files and directories
    - Shell completions
    - Configuration file
- Invite viewers to try it and contribute feedback
