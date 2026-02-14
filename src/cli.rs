use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "crabai", version, about = "Minimal Unix-native multi-provider LLM CLI")]
pub struct Cli {
    /// Prompt name (from prompts directory)
    pub prompt_name: Option<String>,

    /// Override provider
    #[arg(short = 'p', long = "provider")]
    pub provider: Option<String>,

    /// Override model
    #[arg(short = 'm', long = "model")]
    pub model: Option<String>,

    /// Set temperature
    #[arg(short = 't', long = "temperature")]
    pub temperature: Option<f32>,

    /// Set max tokens
    #[arg(long = "max-tokens")]
    pub max_tokens: Option<u32>,

    /// Custom config path
    #[arg(short = 'c', long = "config")]
    pub config: Option<String>,

    /// List supported providers
    #[arg(long = "list-providers", alias = "lp")]
    pub list_providers: bool,

    /// List available prompts
    #[arg(long = "list-prompts", alias = "lpr")]
    pub list_prompts: bool,

    /// List models for a provider
    #[arg(long = "list-models", alias = "lm")]
    pub list_models: bool,

    /// Used with --list-models to query all providers
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Verbose logging
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}
