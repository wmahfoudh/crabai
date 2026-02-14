# CrabAI

Minimal Unix-native multi-provider LLM CLI written in Rust. Single static binary. STDIN in, model output out. Composable in shell pipelines. No Python. No orchestration framework. No hidden state.


## Philosophy

CrabAI is an infrastructure-level tool. It sends a prompt to an LLM provider and prints the response. Nothing more.

- One static Rust binary
- STDIN to model to STDOUT
- Explicit provider separation, each provider isolated in its own module
- Configuration via file with CLI overrides
- Deterministic execution, same input produces same request
- Zero hidden state between invocations
- Designed for shell pipelines and automation

CrabAI is not an agent framework, not a workflow engine, not a chat UI, and not a stateful assistant.


## Features

- 8 LLM providers with isolated implementations
- Model discovery via provider APIs with local caching
- Static fallback model lists for providers without API keys
- Markdown prompt templates loaded from a configurable directory
- STDIN piping with automatic detection
- TOML configuration file with CLI override precedence
- Structured error output to STDERR with exit code 1 on failure
- Raw model output to STDOUT with no formatting or metadata wrapping


## Supported Providers

| Provider | API Style | Environment Variable |
|------------|---------------------|--------------------------|
| OpenAI | OpenAI native | `OPENAI_API_KEY` |
| Anthropic | Anthropic Messages | `ANTHROPIC_API_KEY` |
| Google | Gemini | `GOOGLE_API_KEY` |
| OpenRouter | OpenAI-compatible | `OPENROUTER_API_KEY` |
| Groq | OpenAI-compatible | `GROQ_API_KEY` |
| Together | OpenAI-compatible | `TOGETHER_API_KEY` |
| Mistral | OpenAI-compatible | `MISTRAL_API_KEY` |
| DeepSeek | OpenAI-compatible | `DEEPSEEK_API_KEY` |

OpenAI-compatible providers share a common request/response layer internally. Anthropic and Google have custom implementations matching their respective APIs.


## Installation

Build from source:

```
cargo build --release
```

The binary is at `target/release/crabai`. Copy it to a directory on your PATH.

Install directly via cargo:

```
cargo install --path .
```


## Configuration

Default config path:

```
~/.config/crabai/config.toml
```

### Interactive Configuration

CrabAI provides an interactive configuration tool to create or edit your config file.

**Manual invocation:**

```
crabai --config
```

or

```
crabai -c
```

**Automatic prompt:**

When you run CrabAI without a configured provider, it will automatically offer to create a configuration file:

```
$ crabai summarize
No provider configured.
Would you like to create a configuration file now? (y/n)
```

The interactive tool will guide you through:
- Selecting a default provider
- Choosing a default model (fetched from the provider's API)
- Setting temperature and max tokens
- Configuring the prompts directory
- Setting model cache options
- Customizing API key environment variable names (advanced)

### Using a Custom Config File

Specify a custom config file path with the `--use-config` flag:

```
crabai --use-config ./my-config.toml summarize
```

or

```
crabai -u ./my-config.toml summarize
```

If the config file does not exist, CrabAI uses internal defaults and proceeds without error.

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

**Note:** The `[advanced.api_key_vars]` section is automatically created with default values for all providers. You can manually edit this file to customize environment variable names without re-running the configuration wizard.

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

### Advanced Configuration

The `[advanced]` section is automatically generated with default values for all providers. This allows customization of provider-specific settings.

#### Custom API Key Environment Variables

The configuration file always includes an `[advanced.api_key_vars]` section with default environment variable names for all providers:

```toml
[advanced.api_key_vars]
openai = "OPENAI_API_KEY"
anthropic = "ANTHROPIC_API_KEY"
google = "GOOGLE_API_KEY"
# ... all 8 providers included by default
```

You can customize these values either:
1. **During setup**: Answer "yes" when prompted "Configure advanced settings?"
2. **Manual editing**: Edit the config file directly after creation

**Example customization:**
```toml
[advanced.api_key_vars]
openai = "MY_CUSTOM_OPENAI_KEY"
anthropic = "MY_ANTHROPIC_KEY"
```

This is useful when:
- Your environment uses non-standard variable names
- You're using multiple API keys for the same provider
- You want consistent naming across different tools
- Provider conventions change over time

**Important:** The program always reads these mappings when looking up API keys, so you can change environment variable names without modifying code.

### Resolution Precedence

1. CLI flags (highest priority)
2. Config file
3. Internal defaults


## Environment Variables

Each provider requires its API key set as an environment variable. If the key is missing when a request is made, CrabAI exits with an error:

```
Error: Missing API key for provider: openai
```

Model listing for providers with static fallback lists (Anthropic, Google, DeepSeek) works without an API key. All other providers require a key for model listing.


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

Send piped content through a prompt:

```
cat report.txt | crabai summarize -p openai -m gpt-4.1
```

Send a prompt without STDIN:

```
crabai explain -p anthropic -m claude-sonnet-4-5-20250514
```

Override temperature:

```
echo "Review this code" | crabai summarize -p groq -m llama-3.3-70b-versatile -t 0.5
```

Override max tokens:

```
cat novel.txt | crabai summarize -p openai -m gpt-4.1 --max-tokens 8192
```

List supported providers:

```
crabai --list-providers
```

List models for a specific provider:

```
crabai --list-models -p openai
```

List models for all providers:

```
crabai --list-models -a
```

Output format for `--list-models -a` is `provider:model_name`:

```
openai:gpt-4.1
openai:gpt-4o
anthropic:claude-sonnet-4-5-20250514
google:gemini-2.5-pro
```

List available prompts:

```
crabai --prompt-list
```

Verbose mode prints metadata to STDERR:

```
cat file.txt | crabai summarize -p openai -m gpt-4.1 -v
```

```
Provider: openai
Model: gpt-4.1
Temperature: 0.2
Max tokens: 4096
```

Use a custom config file:

```
crabai -u ./project-config.toml summarize -p anthropic
```

Create or edit configuration interactively:

```
crabai --config
```


## Prompt System

Prompts are Markdown files stored in the prompts directory. The default location is:

```
~/.config/crabai/prompts/
```

This is configurable via the `prompts_dir` key in `config.toml`.

### Bundled Prompts

CrabAI ships with sample prompts embedded in the binary:
- `blog.md` - Blog post writing assistant
- `clean.md` - Code cleaning and refactoring
- `ideas.md` - Idea generation and brainstorming
- `pragmatica.md` - Pragmatic code review
- `weaver.md` - Content weaving and synthesis

These prompts are **automatically installed** to your prompts directory on first run. If you delete a bundled prompt, it will be reinstalled the next time you run CrabAI. This ensures you always have working examples to start with.

**Adding new bundled prompts**: Any `.md` file you add to `src/prompts/` will be automatically embedded in the binary at compile time and deployed with CrabAI. No code changes needed - just add your prompt file and rebuild!

### Prompt File Format

Each prompt is a plain `.md` file. The filename without extension is the prompt name used on the command line.

File `~/.config/crabai/prompts/summarize.md`:

```
Summarize the following text concisely.
```

Invoked as:

```
cat document.txt | crabai summarize -p openai -m gpt-4.1
```

### Prompt Assembly

When STDIN is present, the final message sent to the model is:

```
<prompt file content>

-----

<stdin content>
```

When STDIN is not present, only the prompt file content is sent.

STDIN detection uses the `atty` crate to check whether standard input is a terminal or a pipe.


## Model Discovery

CrabAI fetches model lists dynamically from provider APIs whenever possible. All providers support dynamic model discovery:

- **Primary method**: Queries the provider's models API endpoint
- **Fallback**: Uses built-in static list only when API call fails or no API key is available

This ensures you always have access to the latest models without needing to update CrabAI.

### Provider Model Discovery

| Provider | API Endpoint | Fallback | Notes |
|----------|--------------|----------|-------|
| OpenAI | ✅ | ✅ | Returns gpt-4o, gpt-4-turbo, etc. |
| Anthropic | ✅ | ✅ | Returns Claude 4 & 3.5 models |
| Google | ✅ | ✅ | Returns Gemini 2.0 & 1.5 models |
| OpenRouter | ✅ | ✅ | Returns popular models across providers |
| Groq | ✅ | ✅ | Returns Llama 3.x & Mixtral models |
| Together | ✅ | ✅ | Returns Llama 3.x & open models |
| Mistral | ✅ | ✅ | Returns Mistral Large/Medium/Small |
| DeepSeek | ✅ | ✅ | Returns deepseek-chat & reasoner |

**All providers** now support graceful fallback! If the API is unavailable or no API key is configured, CrabAI returns a curated static list of popular models for that provider.

### Model Cache

Model lists are cached locally to avoid repeated API calls.

Cache location:

```
~/.config/crabai/model_cache.json
```

Cache format is JSON with per-provider entries containing a Unix timestamp and model list. Entries expire after the configured TTL (default 24 hours).

Disable caching in config:

```toml
model_cache = false
```


## Execution Flow

1. Parse CLI arguments
2. Load config file (custom path or default, graceful fallback if missing)
3. If `--list-providers`: print provider names and exit
4. If `--list-prompts`: print prompt names from prompts directory and exit
5. If `--list-models`: resolve provider(s), check cache, fetch if needed, print and exit
6. Resolve provider (CLI flag, then config, then error)
7. Resolve model (CLI flag, then config, then error)
8. Load prompt template file
9. Read STDIN if piped input detected
10. Assemble final prompt (template + separator + stdin, or template only)
11. Resolve temperature and max tokens (CLI flag, then config, then defaults)
12. Send request to provider
13. Print raw response to STDOUT

Errors print to STDERR and exit with code 1. Success exits with code 0.


## Extending CrabAI

To add a new provider:

1. Create `src/providers/<name>.rs`
2. Implement the `Provider` trait:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn send(
        &self,
        model: &str,
        prompt: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> Result<String, CrabError>;

    async fn list_models(&self) -> Result<Vec<String>, CrabError>;

    fn name(&self) -> &str;
}
```

3. Add the variant to `ProviderName` in `src/types.rs`
4. Register the module and factory match arm in `src/providers/mod.rs`
5. Document the required environment variable

For OpenAI-compatible APIs, use the shared helpers in `src/providers/openai_compat.rs` to avoid duplicating request/response handling.


## Project Structure

```
crabai/
  Cargo.toml
  LICENSE
  README.md
  src/
    main.rs
    cli.rs
    config.rs
    error.rs
    types.rs
    prompt_loader.rs
    model_cache.rs
    providers/
      mod.rs
      trait.rs
      openai_compat.rs
      openai.rs
      anthropic.rs
      google.rs
      openrouter.rs
      groq.rs
      together.rs
      mistral.rs
      deepseek.rs
```


## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the full license text.
