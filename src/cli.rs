use clap::{builder::Styles, Parser};

fn styles() -> Styles {
    Styles::styled()
        .header(
            clap::builder::styling::AnsiColor::Yellow
                .on_default()
                .bold(),
        )
        .usage(
            clap::builder::styling::AnsiColor::Yellow
                .on_default()
                .bold(),
        )
        .literal(clap::builder::styling::AnsiColor::Green.on_default().bold())
        .placeholder(clap::builder::styling::AnsiColor::Cyan.on_default())
}

/// Command-line interface definition for CrabAI.
/// Parsed by clap from command-line arguments.
#[derive(Parser, Debug)]
#[command(
    name = "crabai",
    version,
    about = "Minimal Unix-native multi-provider LLM CLI",
    styles = styles()
)]
pub struct Cli {
    /// All remaining arguments after the options.
    /// The first argument might be a prompt name.
    /// The rest are parts of the prompt.
    pub args: Vec<String>,

    /// Model to use (e.g. 'anthropic:claude-3-opus'). Overrides config.
    #[arg(short = 'm', long = "model")]
    pub model: Option<String>,

    /// Sampling temperature (0.0 to 2.0).
    #[arg(short = 't', long = "temperature")]
    pub temperature: Option<f32>,

    /// Maximum tokens in the model's response (or "max").
    #[arg(short = 'T', long = "max-tokens")]
    pub max_tokens: Option<String>,

    /// Path to a custom config file.
    #[arg(short = 'u', long = "use-config")]
    pub use_config: Option<String>,

    /// Launch interactive configuration wizard.
    #[arg(short = 'c', long = "config")]
    pub config: bool,

    /// List all available prompt templates and exit.
    #[arg(short = 'L', long = "list-prompts")]
    pub list_prompts: bool,

    /// List models and copy selection to clipboard.
    #[arg(short = 'l', long = "list-models")]
    pub list_models: bool,

    /// Print request metadata to STDERR.
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}
