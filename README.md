# CrabAI

Minimal multi-provider LLM CLI. Single static binary. STDIN in, model output out. Composable in shell pipelines.

## Context & Philosophy

I'm a big fan of [Fabric](https://github.com/danielmiessler/fabric) by Daniel Miessler, and I've had the privilege of contributing to the project. Fabric pioneered the concept of using AI from the terminal for everyday tasks. However, as Fabric grew, it became increasingly complex, in both good and bad ways.

CrabAI is inspired by Fabric's core idea but takes a minimalist approach. It's designed to be a small, self-contained binary that does one thing well: send a prompt to an LLM and return the response.

*CrabAI was developed to address my specific needs, and while it looks for me reliable, bugs may still occur. So If you notice a problem, please open an issue, I’ll do my best to address it when time permits.*

**Design principles:**
- One static Rust binary, no dependencies
- STDIN to model to STDOUT, designed for Unix pipelines
- Configuration via simple TOML file with CLI overrides
- 8 LLM providers with isolated, maintainable implementations

**What CrabAI is NOT:**
- Not an agent framework
- Not a stateful chat UI (you can't use it to run conversations with LLMs)

CrabAI is terminal tool for quick, composable AI tasks or for running structured prompts with predictable outputs. This can be useful in a production environment (in a cloud function for example)

## Features

- **8 LLM providers**: OpenAI, Anthropic, Google, OpenRouter, Groq, Together, Mistral, DeepSeek
- **Unified Model Selection**: Single `-m provider:model` flag. Automatically handles specific constraints for reasoning models (e.g., OpenAI `o1`/`o3`, `deepseek-reasoner`).
- **Self-Healing Capability Cache**: Tries to learn model limits and parameter names dynamically from provider API metadata and error messages.
- **Interactive Model Lister**: Fuzzy-searchable list of all available models that copies your selection to the clipboard.
- **Markdown Prompts**: Template system with reusable sample prompts just like [Fabric](https://github.com/danielmiessler/fabric).
- **Universal Argument & Pipeline Support**: Construct prompts from arguments, files, and piped `STDIN`.
- **Interactive Configuration**: Step-by-step setup wizard with auto-configuration on first run.

## Supported Providers

| Provider | Environment Variable |
|------------|--------------------------|
| OpenAI | `OPENAI_API_KEY` |
| Anthropic | `ANTHROPIC_API_KEY` |
| Google | `GEMINI_API_KEY` |
| OpenRouter | `OPENROUTER_API_KEY` |
| Groq | `GROQ_API_KEY` |
| Together | `TOGETHER_API_KEY` |
| Mistral | `MISTRAL_API_KEY` |
| DeepSeek | `DEEPSEEK_API_KEY` |

All environment variable names are customizable via the config file. This is useful when other tools impose a different name and you want to avoid conflicts.

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
- Default model selection (with the same interactive lister)
- Temperature, max tokens, and prompts directory
- Model cache settings
- Custom API key environment variable names (optional)

Tip: Use `-u` to specify a custom config file: `crabai -u ./my-config.toml summarize "some text"`

### Config File Example

```toml
# The default model to use, in provider:model format
default_model = "openai:gpt-4o"
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
google = "GEMINI_API_KEY"
openrouter = "OPENROUTER_API_KEY"
groq = "GROQ_API_KEY"
together = "TOGETHER_API_KEY"
mistral = "MISTRAL_API_KEY"
deepseek = "DEEPSEEK_API_KEY"
```

### Config Keys

| Key | Type | Default | Description |
|--------------------------|---------|---------|-----------------------------------------------|
| `default_model` | string | none | Model used when `-m` is not specified, in `provider:model` format. |
| `temperature` | float | `0.2` | Sampling temperature |
| `max_tokens` | integer | `4096` | Maximum tokens in response |
| `prompts_dir` | string | `~/.config/crabai/prompts` | Directory containing prompt templates |
| `model_cache` | boolean | `true` | Enable or disable model list caching |
| `model_cache_ttl_hours` | integer | `24` | Hours before cached model lists expire |
| `advanced.api_key_vars` | table | (see below) | Custom environment variable names for API keys |

The `prompts_dir` value supports `~/` expansion.

**Setting precedence:** CLI flags > config file > internal defaults

## CLI Usage

The core principle is that everything after the options (`-m`, `-t`, etc.) is part of the prompt.

```
crabai [OPTIONS] [ARG_0] [ARG_1] ... [ARG_N]
```

### Prompt Assembly Logic

CrabAI builds the final prompt using these parts, in order:
1.  **Prompt File**: If `ARG_0` matches the name of a prompt file in your `prompts_dir` (e.g., `summarize`), the content of that file is used as the base.
2.  **Literal Prompt**: If `ARG_0` is not a prompt file, it's treated as the literal text for the prompt.
3.  **Additional Arguments**: All subsequent arguments (`ARG_1` to `ARG_N`) are appended to the prompt, separated by newlines.
4.  **STDIN**: If content is piped to `crabai`, it is appended to the very end.

This provides a powerful and flexible way to compose prompts.

### Flags

| Long Flag | Short | Description |
|-------------------|-------|----------------------------------------------|
| `--model` | `-m` | Set model, in `provider:model` format (e.g., `anthropic:claude-3-opus`) |
| `--temperature` | `-t` | Set sampling temperature |
| `--max-tokens` | `-T` | Set max tokens (use `max` for model limit) |
| `--use-config` | `-u` | Path to custom config file |
| `--config` | `-c` | Launch the interactive config wizard |
| `--list-prompts` | `-L` | List available prompt templates |
| `--list-models` | `-l` | Show an interactive list of all models to copy to clipboard |
| `--verbose` | `-v` | Print request metadata to STDERR |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version |

### Examples

```bash
# 1. Use a prompt file and pipe content to it
# (Assumes 'summarize.md' exists in your prompts directory)
cat report.txt | crabai summarize -m openai:gpt-4o

# 2. Send a direct, one-line prompt
crabai "Translate 'hello world' to French"

# 3. Build a multi-part prompt from arguments
crabai "Analyze this code for bugs:" "$(cat code.py)" "Focus on security issues."

# 4. List models, find one, and use it
crabai --list-models  # Interactively search and copy 'groq:gemma2-9b-it'
cat story.txt | crabai -m groq:gemma2-9b-it "Summarize this story in one sentence."

# 5. Verbose mode (prints metadata to STDERR)
cat file.txt | crabai summarize -v
```

## Prompts

Prompts are Markdown files in `~/.config/crabai/prompts/`. Sample prompts are included and auto-installed on first run.

**Format:** Plain `.md` files. Filename (without extension) becomes the prompt name.

```bash
echo "Your prompt text" > ~/.config/crabai/prompts/summarize.md
cat document.txt | crabai summarize
```

## Model Capabilities & Discovery

CrabAI fetches model lists and capabilities (token limits, parameter support) dynamically from provider APIs. 

**Self-Healing Logic:** If an API request fails due to a limit mismatch or unsupported parameter, CrabAI parses the error message, tries to "learn" the correct constraint from the error message, and updates the local cache if possible.

**Caching:** Model info is cached at `~/.config/crabai/model_cache.json`. A bundled seed cache provides immediate support many common models on first run.

## How It Works

1. Parse CLI → Load config → Handle list commands (if any) → Exit
2. Resolve provider & model from `-m` flag or config → Assemble prompt from args and STDIN
3. Send to LLM → Print response to STDOUT
4. Errors go to STDERR with exit code 1

## Extending

**Add a provider:** Create `src/providers/<name>.rs`, implement the `Provider` trait, register in `src/providers/mod.rs` and `src/types.rs`. Use `openai_compat.rs` helpers for OpenAI-compatible APIs.

**Add prompts:** Place `.md` files in `src/prompts/` and rebuild. They'll be embedded and auto-installed.


## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the full license text.
