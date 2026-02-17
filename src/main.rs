mod bundled_prompts;
mod cli;
mod config;
mod config_editor;
mod error;
mod model_cache;
mod prompt_loader;
mod providers;
mod types;

use std::io::Read;
use std::process;

use clap::Parser;

use bundled_prompts::BundledPrompts;
use cli::Cli;
use config::Config;
use error::CrabError;
use model_cache::ModelCache;
use providers::{get_provider_with_config, list_provider_names};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

/// Main execution flow for CrabAI.
///
/// Processing order:
/// 1. Interactive config mode (--config) - launches wizard and exits
/// 2. List commands (--list-models, --list-prompts) - print and exit
/// 3. Normal operation - resolve provider/model/prompt and send request
///
/// Resolution precedence for all settings: CLI flags > config file > defaults
async fn run(cli: Cli) -> Result<(), CrabError> {
    // Interactive config mode: launch wizard and exit
    if cli.config {
        return config_editor::run_interactive_config(cli.use_config.as_deref()).await;
    }

    let config = Config::load(cli.use_config.as_deref())?;

    // Auto-install bundled prompts if any are missing
    let prompts_dir = config.prompts_dir();
    if BundledPrompts::has_missing_prompts(&prompts_dir) {
        match BundledPrompts::install_to(&prompts_dir) {
            Ok(count) if count > 0 => {
                if cli.verbose {
                    eprintln!(
                        "Installed {} bundled prompt(s) to {}",
                        count,
                        prompts_dir.display()
                    );
                }
            }
            Ok(_) => {} // No new prompts to install
            Err(e) => {
                if cli.verbose {
                    eprintln!("Warning: Could not install bundled prompts: {}", e);
                }
            }
        }
    }

    if cli.list_prompts {
        let prompts_dir = config.prompts_dir();
        let prompts = prompt_loader::list_prompts(&prompts_dir)?;
        for p in prompts {
            println!("{p}");
        }
        return Ok(());
    }

    if cli.list_models {
        return list_models(&cli, &config).await;
    }

    // Extract provider and model from --model flag or config
    let (provider_name, model_name) = match cli.model.or(config.default_model.clone()) {
        Some(model_str) => {
            if model_str.contains(':') {
                let parts: Vec<&str> = model_str.split(':').collect();
                (parts[0].to_string(), parts[1].to_string())
            } else {
                // If no provider is specified, use the default provider from config
                let provider = config.default_provider.clone().ok_or_else(|| {
                    CrabError::ConfigError(
                        "No provider specified in model and no default provider in config."
                            .to_string(),
                    )
                })?;
                (provider, model_str)
            }
        }
        None => {
            // No model or provider configured - offer to create config
            eprintln!("No model configured.");
            eprintln!("Would you like to create a configuration file now? (y/n)");

            let mut response = String::new();
            std::io::stdin().read_line(&mut response)?;

            if response.trim().to_lowercase().starts_with('y') {
                config_editor::run_interactive_config(cli.use_config.as_deref()).await?;
                eprintln!("\nConfiguration created! Please run crabai again.");
                return Ok(());
            } else {
                return Err(CrabError::ConfigError(
                    "No model specified. Use -m <provider:model> or run 'crabai --config' to configure."
                        .to_string(),
                ));
            }
        }
    };

    let provider = get_provider_with_config(&provider_name, &config)?;

    // Handle prompt assembly
    let (prompt_content, remaining_args) = if !cli.args.is_empty() {
        let prompts_dir = config.prompts_dir();
        let first_arg = &cli.args[0];
        match prompt_loader::load_prompt(first_arg, &prompts_dir) {
            Ok(content) => (content, &cli.args[1..]), // First arg is a prompt name
            Err(_) => (first_arg.to_string(), &cli.args[1..]), // First arg is a literal prompt
        }
    } else {
        // No arguments, check for stdin
        ("".to_string(), &[][..])
    };

    // Only read STDIN when input is piped (not a TTY).
    let stdin_content = if !atty::is(atty::Stream::Stdin) {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Some(buf)
    } else {
        None
    };

    let final_prompt =
        prompt_loader::assemble(&prompt_content, remaining_args, stdin_content.as_deref());

    if final_prompt.trim().is_empty() {
        return Err(CrabError::ConfigError(
            "Prompt is empty. Provide a prompt as an argument or pipe content from stdin."
                .to_string(),
        ));
    }

    let temperature = cli
        .temperature
        .unwrap_or_else(|| config.resolve_temperature());
    let max_tokens = cli
        .max_tokens
        .unwrap_or_else(|| config.resolve_max_tokens());

    if cli.verbose {
        eprintln!("Provider: {}", provider.name());
        eprintln!("Model: {model_name}");
        eprintln!("Temperature: {temperature}");
        eprintln!("Max tokens: {max_tokens}");
    }

    let response = provider
        .send(&model_name, &final_prompt, temperature, max_tokens)
        .await?;

    // Raw output only; no trailing newline beyond what the model returns.
    print!("{response}");

    Ok(())
}

/// Handles the --list-models command.
/// Fetches models for all providers, displays an interactive list,
/// and copies the selected model identifier to the clipboard.
async fn list_models(cli: &Cli, config: &Config) -> Result<(), CrabError> {
    let config_dir = Config::config_dir();
    let mut cache = ModelCache::load(&config_dir);
    let ttl = config.cache_ttl_hours();
    let cache_enabled = config.model_cache_enabled();

    let mut all_models = Vec::new();
    for name in list_provider_names() {
        let models = get_models(name, &mut cache, ttl, cache_enabled, config).await;
        match models {
            Ok(models) => {
                for m in models {
                    all_models.push(format!("{name}:{m}"));
                }
            }
            Err(e) => {
                if cli.verbose {
                    eprintln!("Warning: {name}: {e}");
                }
            }
        }
    }

    if cache_enabled {
        let _ = cache.save(&config_dir);
    }

    if all_models.is_empty() {
        return Err(CrabError::ConfigError(
            "Could not fetch any models. Check your API keys and network connection.".to_string(),
        ));
    }

    all_models.sort();

    let selection = dialoguer::FuzzySelect::new()
        .with_prompt("Select a model to copy to clipboard")
        .items(&all_models)
        .default(0)
        .interact_opt()?;

    if let Some(index) = selection {
        let selected_model: &String = &all_models[index];
        // Use a clipboard crate to copy to clipboard
        use clipboard::{ClipboardContext, ClipboardProvider};
        let mut ctx: ClipboardContext = ClipboardProvider::new()?;
        ctx.set_contents(selected_model.to_string())?;
        eprintln!("Copied '{selected_model}' to clipboard.");
    }

    Ok(())
}

/// Retrieves model list for a provider, using cache if available and valid.
async fn get_models(
    provider_name: &str,
    cache: &mut ModelCache,
    ttl: u64,
    cache_enabled: bool,
    config: &Config,
) -> Result<Vec<String>, CrabError> {
    if cache_enabled {
        if let Some(cached) = cache.get(provider_name, ttl) {
            return Ok(cached.to_vec());
        }
    }

    let provider = get_provider_with_config(provider_name, config)?;
    let models = provider.list_models().await?;

    if cache_enabled {
        cache.set(provider_name, models.clone());
    }

    Ok(models)
}
