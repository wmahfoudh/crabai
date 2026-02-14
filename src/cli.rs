use clap::Parser;

/// Command-line interface definition for CrabAI.
/// Parsed by clap from command-line arguments.
#[derive(Parser, Debug)]
#[command(name = "crabai", version, about = "Minimal Unix-native multi-provider LLM CLI")]
pub struct Cli {
    /// Name of the prompt template to use (without .md extension).
    /// The prompt file is loaded from the prompts directory.
    pub prompt_name: Option<String>,

    /// Provider to use for the request.
    /// Overrides the default_provider from config.
    #[arg(short = 'p', long = "provider")]
    pub provider: Option<String>,

    /// Model to use for the request.
    /// Overrides the default_model from config.
    #[arg(short = 'm', long = "model")]
    pub model: Option<String>,

    /// Sampling temperature (0.0 to 2.0).
    /// Overrides the temperature from config.
    #[arg(short = 't', long = "temperature")]
    pub temperature: Option<f32>,

    /// Maximum tokens in the model's response.
    /// Overrides the max_tokens from config.
    #[arg(short = 'T', long = "max-tokens")]
    pub max_tokens: Option<u32>,

    /// Path to a custom config file.
    /// If not specified, uses ~/.config/crabai/config.toml.
    #[arg(short = 'u', long = "use-config")]
    pub use_config: Option<String>,

    /// Launch interactive config creation/editing wizard.
    /// Guides the user through creating or updating the config file.
    #[arg(short = 'c', long = "config")]
    pub config: bool,

    /// List all supported LLM providers and exit.
    #[arg(short = 'P', long = "list-providers")]
    pub list_providers: bool,

    /// List all available prompt templates and exit.
    #[arg(short = 'L', long = "prompt-list")]
    pub list_prompts: bool,

    /// List available models for a provider and exit.
    /// Requires -p <provider> unless -a is used.
    #[arg(short = 'M', long = "list-models")]
    pub list_models: bool,

    /// When used with --list-models, query all providers.
    /// Output format: provider:model_name
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Print request metadata to STDERR before sending.
    /// Shows provider, model, temperature, and max_tokens.
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}
