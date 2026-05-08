# moldy

A minimal templating CLI that copies directory templates from a config file to a target path.

## Installation

```bash
cargo install moldy
```

Or build from source:

```bash
git clone <repo-url>
cd moldy
cargo build --release
```

## Configuration

Create a config file at `~/.config/moldy/config.toml`:

```toml
[templates]
react = "/home/user/.moldy-templates/react"
api   = "/home/user/.moldy-templates/api"
cli   = "/home/user/.moldy-templates/cli"
```

Each template key points to an absolute path on disk containing your template files.

## Usage

```
moldy <TARGET_DIRECTORY> <PATH_IN_CONFIG>
```

### Examples

**Case 1: Copy into existing directory**
```bash
# ~/projects exists, creates ~/projects/react/ with template contents
moldy ~/projects react
```

**Case 2: Create new directory with template contents**
```bash
# ~/projects/my-app doesn't exist, creates it with template contents directly
moldy ~/projects/my-app react
```

**Case 3: Error when parent doesn't exist**
```bash
# Error: parent directory doesn't exist (won't create nested missing dirs)
moldy ~/nonexistent1/nonexistent2 react
```

## Features

- Copies templates with `mv`-like semantics
- Preserves file permissions
- Copies symlinks as symlinks (doesn't follow them)
- Prompts before overwriting existing files
- Colored terminal output
- Depth restriction: only creates one new directory level
- Lists available templates when key is not found

## License

MIT
