# Git Extra Commands

An extra useful set of Git related commands. Requires that you have [Git](https://git-scm.com) installed.  Run `git_extra` to see the full list.

| Command  | Description                                                                                                                                                                                                                                                                                   |
| -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `browse` | Browse to the site hosting the `origin` for the current repo.  Uses `git remotes -vv` to determine the correct site to open.  Currently supports [Git](https://github.com), [GitLab](https://gitlab.com), [BitBucket](https://bitbucket.org) or a self hosted [Gitea](https://gitea.io) site. |

## Installation

_The current release is only tested on macOS_.

Download and install with:

```sh
cargo install git_extra
```

You can configure the commands in your `.gitconfig` configuration by running `git config --global --edit` and adding:

```toml
[alias]
browse = !git_extra browse
```

This will allow you to type `git browse` to browse to the repository web page.

## To Do

- Add a `fork` command that will fork and add `origin` and `upstream` remotes
- Add a `set-config` command that updates local config based on a cloud based config
- Add a `quickstart` command that creates a new repo based on the URL of a repo and which optionally runs a post `clone` cofiguration script
