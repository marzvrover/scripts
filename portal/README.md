# portal

Quick switching between oh-my-opencode model providers.

## Usage

```bash
# Switch to GitHub Copilot
portal switch copilot

# Switch to OpenRouter
portal switch openrouter

# Switch to a custom provider
portal switch custom-provider

# Show current configuration
portal status

# List available providers (built-in + custom)
portal list

# Dry run (show changes without applying)
portal --dry-run switch openrouter

# Force create backup (normally only creates on first switch)
portal --backup switch copilot

# Revert to latest backup
portal revert

# Revert to specific backup
portal revert /path/to/backup.json

# Use custom config file
portal --config /path/to/oh-my-opencode.json status
```

## How It Works

Portal reads your `~/.config/opencode/oh-my-opencode.json` and transforms model names between providers.

## Default Model Mappings

| Base Model        | Copilot           | OpenRouter                    |
| ----------------- | ----------------- | ----------------------------- |
| claude-opus-4.5   | claude-opus-4.5   | anthropic/claude-opus-4.5     |
| claude-sonnet-4.5 | claude-sonnet-4.5 | anthropic/claude-sonnet-4.5   |
| gpt-5.2           | gpt-5.2           | openai/gpt-5.2                |
| gpt-4.1           | gpt-4.1           | openai/gpt-4.1                |
| o3                | o3                | openai/o3                     |
| gemini-3-flash    | gemini-3-flash    | google/gemini-3-flash-preview |
| gemini-3-pro      | gemini-3-pro      | google/gemini-3-pro-preview   |

## Custom Providers

Create provider configs in `~/.config/portal/` for custom setups.

```bash
mkdir -p ~/.config/portal
```

### Example: custom-open-router.json

```json
{
  "agents": {
    "sisyphus": { "model": "openrouter/anthropic/claude-opus-4.5" },
    "oracle": { "model": "openrouter/openai/o3" },
    "explore": { "model": "openrouter/google/gemini-3-flash-preview" }
  }
}
```

The format matches oh-my-opencode's agent structure. Any agents not specified will use built-in model mappings.

### Example: google.json

```json
{
  "agents": {
    "sisyphus": { "model": "google/gemini-3-pro-preview" },
    "librarian": { "model": "google/gemini-3-flash-preview" },
    "oracle": { "model": "google/gemini-3-pro-preview" },
    "explore": { "model": "google/gemini-3-flash-preview" }
  }
}
```

Then switch: `portal switch google`

## Backup Behavior

Portal automatically creates a backup **the first time** you switch providers. Subsequent switches won't create backups unless you use `--backup`:

```bash
# First switch - creates backup automatically
portal switch openrouter
# Backup created: oh-my-opencode.json.bak.2026-01-29T15-00-00-000Z

# Second switch - no backup (one already exists)
portal switch copilot

# Force a new backup
portal --backup switch openrouter
```

### Reverting

```bash
# Revert to most recent backup
portal revert

# Revert to a specific backup
portal revert ~/.config/opencode/oh-my-opencode.json.bak.2026-01-29T15-00-00-000Z
```

## Installation

```bash
cd portal
cargo build --release
cp target/release/portal ~/.local/bin/
```

## Requirements

- Rust 1.70+
- oh-my-opencode configured at `~/.config/opencode/oh-my-opencode.json`
