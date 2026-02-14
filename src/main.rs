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
use providers::{list_provider_names, get_provider_with_config};

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
/// 2. List commands (--list-providers, --list-models, --prompt-list) - print and exit
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
                    eprintln!("Installed {} bundled prompt(s) to {}", count, prompts_dir.display());
                }
            }
            Ok(_) => {}, // No new prompts to install
            Err(e) => {
                if cli.verbose {
                    eprintln!("Warning: Could not install bundled prompts: {}", e);
                }
            }
        }
    }

    if cli.list_providers {
        for name in list_provider_names() {
            println!("{name}");
        }
        return Ok(());
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

    // Resolution order: CLI flag > config file > prompt for config creation.
    let provider_name = match cli.provider.or(config.default_provider.clone()) {
        Some(p) => p,
        None => {
            // No provider configured - offer to create config
            eprintln!("No provider configured.");
            eprintln!("Would you like to create a configuration file now? (y/n)");
            
            let mut response = String::new();
            std::io::stdin().read_line(&mut response)?;
            
            if response.trim().to_lowercase().starts_with('y') {
                config_editor::run_interactive_config(cli.use_config.as_deref()).await?;
                eprintln!("\nConfiguration created! Please run crabai again.");
                return Ok(());
            } else {
                return Err(CrabError::ConfigError(
                    "No provider specified. Use -p <provider> or run 'crabai --config' to configure."
                        .to_string(),
                ));
            }
        }
    };
    let provider = get_provider_with_config(&provider_name, &config)?;

    let model = match cli.model.or(config.default_model.clone()) {
        Some(m) => m,
        None => {
            return Err(CrabError::ConfigError(
                "No model specified. Use -m <model> or run 'crabai --config' to configure.".to_string(),
            ));
        }
    };

    let prompt_name = cli.prompt_name.ok_or_else(|| {
        CrabError::ConfigError("No prompt specified. Provide a prompt name as argument.".to_string())
    })?;
    let prompts_dir = config.prompts_dir();
    let prompt_content = prompt_loader::load_prompt(&prompt_name, &prompts_dir)?;

    // Only read STDIN when input is piped (not a TTY).
    let stdin_content = if !atty::is(atty::Stream::Stdin) {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Some(buf)
    } else {
        None
    };

    let final_prompt = prompt_loader::assemble(&prompt_content, stdin_content.as_deref());

    let temperature = cli.temperature.unwrap_or_else(|| config.resolve_temperature());
    let max_tokens = cli.max_tokens.unwrap_or_else(|| config.resolve_max_tokens());

    if cli.verbose {
        eprintln!("Provider: {}", provider.name());
        eprintln!("Model: {model}");
        eprintln!("Temperature: {temperature}");
        eprintln!("Max tokens: {max_tokens}");
    }

    let response = provider
        .send(&model, &final_prompt, temperature, max_tokens)
        .await?;

    // Raw output only; no trailing newline beyond what the model returns.
    print!("{response}");

    Ok(())
}

/// Handles the --list-models command.
/// 
/// Without -a: Lists models for a single provider (requires -p or default_provider).
/// With -a: Lists models for all providers in "provider:model" format.
/// 
/// Uses model cache if enabled. Errors during all-provider listing are silently
/// skipped unless --verbose is set.
async fn list_models(cli: &Cli, config: &Config) -> Result<(), CrabError> {
    let config_dir = Config::config_dir();
    let mut cache = ModelCache::load(&config_dir);
    let ttl = config.cache_ttl_hours();
    let cache_enabled = config.model_cache_enabled();

    if cli.all {
        for name in list_provider_names() {
            let models = get_models(name, &mut cache, ttl, cache_enabled).await;
            match models {
                Ok(models) => {
                    for m in models {
                        println!("{name}:{m}");
                    }
                }
                Err(e) => {
                    if cli.verbose {
                        eprintln!("Warning: {name}: {e}");
                    }
                }
            }
        }
    } else {
        let provider_name = cli
            .provider
            .as_deref()
            .or(config.default_provider.as_deref())
            .ok_or_else(|| {
                CrabError::ConfigError(
                    "Use -p <provider> or -a to list models for all providers.".to_string(),
                )
            })?;

        let models = get_models(provider_name, &mut cache, ttl, cache_enabled).await?;
        for m in models {
            println!("{m}");
        }
    }

    if cache_enabled {
        let _ = cache.save(&config_dir);
    }

    Ok(())
}

/// Retrieves model list for a provider, using cache if available and valid.
/// 
/// Cache behavior:
/// - If cache is enabled and entry is fresh, returns cached list
/// - Otherwise, fetches from provider API and updates cache
/// - Uses default Config for API key environment variable resolution
async fn get_models(
    provider_name: &str,
    cache: &mut ModelCache,
    ttl: u64,
    cache_enabled: bool,
) -> Result<Vec<String>, CrabError> {
    if cache_enabled {
        if let Some(cached) = cache.get(provider_name, ttl) {
            return Ok(cached.to_vec());
        }
    }

    let config = Config::default();
    let provider = get_provider_with_config(provider_name, &config)?;
    let models = provider.list_models().await?;

    if cache_enabled {
        cache.set(provider_name, models.clone());
    }

    Ok(models)
}
