# Git Extra Commands

An extra useful set of Git related commands. Requires that you have [Git](https://git-scm.com) installed.  Run `git-extra` to see the full list.

| Command       | Description                                                                                                                                                                                                                                                                                   |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `browse`      | Browse to the site hosting the `origin` for the current repo.  Uses `git remotes -vv` to determine the correct site to open.  Currently supports [Git](https://github.com), [GitLab](https://gitlab.com), [BitBucket](https://bitbucket.org) or a self hosted [Gitea](https://gitea.io) site. |
| `quick-start` | Quickly start a new project by `git clone` and then running a customize script on the cloned repo (see below)                                                                                                                                                                                 |

## Installation

> The current release is only tested on macOS. In v2 the name of the executable has been changed to `git-extra` from `git_extra` so that the `git` command sees it as a new `extra` command.

Download and install with:

```sh
cargo install git_extra
```

## Git Configuration

Because the tool is name `git-extra` if it's in your path it will show up as a new git command, e.g. you can type `git extra --help`.

You can configure additioal shortcuts in your `.gitconfig` configuration by running `git config --global --edit` and adding:

```toml
[alias]
brw = !git-extra browse
qst = !git-extra quick-start
```

This will allow you to type `git brw` or `git qst` as a shortcut.

## Quick Start

The `quick-start` command does just two things:

1. Clones a repo from a URL into a new directory
2. Runs a customization script

You can specify the URL, e.g. `https`, `ssh:` or `file:`, new directory and customization script on the command line OR, more usefully, use a shortcut name from a local `repos.tomol` file.  To do this, create a `~/.config/git_extra/repos.toml` file, then add all your favorite repo's URL's and descriptions in it. The file format is as follows:

```toml
[rust-cli]
description = "My favorite Rust CLI quickstart repo"
origin = "git@github.com:jlyonsmith/rust-cli-quickstart.git"
customizer = "customize.ts"
```

The TOML table name is the short name for the entry, i.e. `rust-cli` in this  case.  The other fields are:

| Name          | Default     | Description                                                       |
| ------------- | ----------- | ----------------------------------------------------------------- |
| `description` | Empty       | A description for the entry                                       |
| `origin`      | Required    | The `origin` URL of the repo                                      |
| `customizer`  | `customize` | The customization script to run in the root of the cloned project |

The customization script can be written in any scripting language.  The file just needs to be marked as executable, e.g. with `chmod u+x`.  You can also include a `#!` at the start of the script.

> BE CAREFUL!  There are no checks done on the script before running it, so don't `quick-start` from any repo that you haven't carefully examined first.

You can list all your saved repos with `git_extra quick-start --list`.

## To Do

- Add `pull-request` command that creates a pull request from `origin` to `upstream`
- Add `fork` command that will fork an `upstream` and add an `origin` from the command line
- Add `set-config` command that updates local config based on a cloud based config
