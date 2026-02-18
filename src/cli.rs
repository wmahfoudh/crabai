use clap::Parser;

/// Command-line interface definition for CrabAI.
/// Parsed by clap from command-line arguments.
#[derive(Parser, Debug)]
#[command(
    name = "crabai",
    version,
    about = "Minimal Unix-native multi-provider LLM CLI"
)]
pub struct Cli {
    /// All remaining arguments after the options.
    /// The first argument might be a prompt name.
    /// The rest are parts of the prompt.
    pub args: Vec<String>,

    /// Model to use for the request, in provider:model format (e.g., "anthropic:claude-3-opus").
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
    pub max_tokens: Option<String>,

    /// Path to a custom config file.
    /// If not specified, uses ~/.config/crabai/config.toml.
    #[arg(short = 'u', long = "use-config")]
    pub use_config: Option<String>,

    /// Launch interactive config creation/editing wizard.
    /// Guides the user through creating or updating the config file.
    #[arg(short = 'c', long = "config")]
    pub config: bool,

    /// List all available prompt templates and exit.
    #[arg(short = 'L', long = "list-prompts")]
    pub list_prompts: bool,

    /// List available models for all providers and exit.
    /// The user can select a model to copy to the clipboard.
    #[arg(short = 'l', long = "list-models")]
    pub list_models: bool,

    /// Print request metadata to STDERR before sending.
    /// Shows provider, model, temperature, and max_tokens.
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}
