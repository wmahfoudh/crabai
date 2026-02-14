# CrabAI

Minimal Unix-native multi-provider LLM CLI written in Rust. Single static binary. STDIN in, model output out. Composable in shell pipelines.


## Context & Philosophy

I'm a big fan of [Fabric](https://github.com/danielmiessler/fabric) by Daniel Miessler, and I've had the privilege of contributing to the project. Fabric pioneered the concept of using AI with structured prompts for everyday tasks. However, as Fabric grew, it became increasingly complex, a "monster" 🙂 in both good and bad ways.

CrabAI is inspired by Fabric's core idea but takes a minimalist approach. It's designed to be an **infrastructure-level tool** that does one thing well: send a prompt to an LLM and return the response.

**Design principles:**
- One static Rust binary, no Python or Node.js dependencies
- STDIN to model to STDOUT, designed for Unix pipelines
- Zero hidden state between invocations
- Configuration via simple TOML file with CLI overrides
- Deterministic execution—same input produces same request
- 8 LLM providers with isolated, maintainable implementations

**What CrabAI is NOT:**
- Not an agent framework
- Not a workflow engine
- Not a chat UI
- Not a stateful assistant

CrabAI is a tool for quick, composable AI tasks in your terminal.


## Features

- **8 LLM providers**: OpenAI, Anthropic, Google, OpenRouter, Groq, Together, Mistral, DeepSeek
- **Dynamic model discovery**: Fetches latest models from provider APIs with local caching
- **Interactive configuration**: Step-by-step setup wizard with auto-configuration on first run
- **Markdown prompts**: Template system with sample prompts included
- **Unix-friendly**: STDIN/STDOUT piping, deterministic output, exit codes
- **Customizable**: TOML config with CLI overrides, custom API key env vars


## Supported Providers

| Provider | Environment Variable |
|------------|--------------------------|
| OpenAI | `OPENAI_API_KEY` |
| Anthropic | `ANTHROPIC_API_KEY` |
| Google | `GOOGLE_API_KEY` |
| OpenRouter | `OPENROUTER_API_KEY` |
| Groq | `GROQ_API_KEY` |
| Together | `TOGETHER_API_KEY` |
| Mistral | `MISTRAL_API_KEY` |
| DeepSeek | `DEEPSEEK_API_KEY` |

All environment variable names are customizable via the config file.


## Installation

```bash
cargo build --release
# Binary at target/release/crabai

# Or install directly
cargo install --path .
```


## Configuration

Default config: `~/.config/crabai/config.toml`

### Interactive Setup

Run `crabai --config` (or `-c`) to launch the configuration wizard, or it will prompt automatically on first use.

The wizard guides you through:
- Provider and model selection (with live API model lists)
- Temperature, max tokens, and prompts directory
- Model cache settings
- Custom API key environment variable names (optional)

Use `-u` to specify a custom config file: `crabai -u ./my-config.toml summarize`

### Config File Example

```toml
default_provider = "openai"
default_model = "gpt-4o"
temperature = 0.2
max_tokens = 4096
prompts_dir = "~/.config/crabai/prompts"
model_cache = true
model_cache_ttl_hours = 24

# Advanced configuration is always generated with defaults
# Edit these values to customize API key environment variable names
[advanced.api_key_vars]
openai = "OPENAI_API_KEY"
anthropic = "ANTHROPIC_API_KEY"
google = "GOOGLE_API_KEY"
openrouter = "OPENROUTER_API_KEY"
groq = "GROQ_API_KEY"
together = "TOGETHER_API_KEY"
mistral = "MISTRAL_API_KEY"
deepseek = "DEEPSEEK_API_KEY"
```



### Config Keys

| Key | Type | Default | Description |
|--------------------------|---------|---------|-----------------------------------------------|
| `default_provider` | string | none | Provider used when `-p` is not specified |
| `default_model` | string | none | Model used when `-m` is not specified |
| `temperature` | float | `0.2` | Sampling temperature |
| `max_tokens` | integer | `4096` | Maximum tokens in response |
| `prompts_dir` | string | `~/.config/crabai/prompts` | Directory containing prompt templates |
| `model_cache` | boolean | `true` | Enable or disable model list caching |
| `model_cache_ttl_hours` | integer | `24` | Hours before cached model lists expire |
| `advanced.api_key_vars` | table | (see below) | Custom environment variable names for API keys |

The `prompts_dir` value supports `~/` expansion.

The `[advanced.api_key_vars]` section lets you customize environment variable names if your setup uses non-standard names.

**Setting precedence:** CLI flags > config file > internal defaults


## CLI Usage

```
crabai [OPTIONS] [PROMPT_NAME]
```

### Arguments

| Argument | Required | Description |
|--------------|----------|----------------------------------------------|
| `PROMPT_NAME` | For send | Name of prompt template (without `.md` extension) |

### Flags

| Long Flag | Short | Description |
|-------------------|-------|----------------------------------------------|
| `--provider` | `-p` | Override provider |
| `--model` | `-m` | Override model |
| `--temperature` | `-t` | Set sampling temperature |
| `--max-tokens` | `-T` | Set max tokens |
| `--use-config` | `-u` | Path to custom config file |
| `--config` | `-c` | Interactive config creation/editing |
| `--list-providers` | `-P` | List all supported providers |
| `--prompt-list` | `-L` | List available prompt templates |
| `--list-models` | `-M` | List models for a provider |
| `--all` | `-a` | With `--list-models`, query all providers |
| `--verbose` | `-v` | Print request metadata to STDERR |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version |

### Examples

```bash
# Send piped content through a prompt
cat report.txt | crabai summarize -p openai -m gpt-4o

# List providers and models
crabai -P
crabai -M -p anthropic
crabai -M -a  # All providers (format: provider:model_name)

# Override settings
echo "Review this code" | crabai review -p groq -t 0.5 -T 2000

# Verbose mode (prints metadata to STDERR)
cat file.txt | crabai summarize -v
```


## Prompts

Prompts are Markdown files in `~/.config/crabai/prompts/`. Sample prompts are included and auto-installed on first run.

**Format:** Plain `.md` files. Filename (without extension) becomes the prompt name.

**Assembly:** When STDIN is present, the prompt and input are combined with a separator. Without STDIN, only the prompt is sent.

```bash
echo "Your prompt text" > ~/.config/crabai/prompts/summarize.md
cat document.txt | crabai summarize -p openai -m gpt-4o
```


## Model Discovery

CrabAI fetches model lists dynamically from provider APIs with graceful fallback to static lists if API is unavailable. Model lists are cached locally (default 24hrs) at `~/.config/crabai/model_cache.json`.


## How It Works

1. Parse CLI → Load config → Handle list commands (if any) → Exit
2. Resolve provider & model → Load prompt → Read STDIN (if piped)
3. Send to LLM → Print response to STDOUT
4. Errors go to STDERR with exit code 1


## Extending

**Add a provider:** Create `src/providers/<name>.rs`, implement the `Provider` trait, register in `src/providers/mod.rs` and `src/types.rs`. Use `openai_compat.rs` helpers for OpenAI-compatible APIs.

**Add prompts:** Place `.md` files in `src/prompts/` and rebuild. They'll be embedded and auto-installed.


## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the full license text.
