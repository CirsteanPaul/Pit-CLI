# Pit - CLI Tool for Version Control

## Description

Pit is a command-line interface (CLI) tool for version control, providing core commands similar to Git (init, add, commit, merge, etc). While Pit uses the same file structure and caching approach as Git, it may not compress blobs as efficiently. It includes its own `.pitignore` file.

This project is my first venture into Rust, and while there's room for improvement, I'm proud of what I've accomplished.

## Languages and Utilities Used

- Rust 🦀

## Available Commands

In the project directory, you can use the following commands:

### `pit init`

Initializes the `.pit` folder for version control.

### `pit add`

Adds the current version of files/directories in the system as blobs.

### `pit commit -m "message"`

Creates a snapshot of the current file tree with the specified message.

### `pit checkout name create`

Creates or moves to the branch with the given name. The `create` variable is a boolean; setting it to `true` will create the branch if it doesn't exist.

### `pit diff commit/file`

Generates a visual representation of the differences between the current system version and a specific commit or file.

### `pit status`

Displays the current files that are added, modified, or deleted compared to the last snapshot.

### `pit merge`

Merges branches. In case of conflicts, it breaks and alerts the user. The next step should be using `pit diff` to resolve conflicts.
