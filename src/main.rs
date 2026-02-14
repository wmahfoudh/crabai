mod cli;
mod config;
mod error;
mod model_cache;
mod prompt_loader;
mod providers;
mod types;

use std::io::Read;
use std::process;

use clap::Parser;

use cli::Cli;
use config::Config;
use error::CrabError;
use model_cache::ModelCache;
use providers::list_provider_names;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

/// Main execution flow. List commands exit early; otherwise resolves
/// provider, model, and prompt, then sends a single request.
async fn run(cli: Cli) -> Result<(), CrabError> {
    let config = Config::load(cli.config.as_deref())?;

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

    // Resolution order: CLI flag > config file > error.
    let provider_name = cli
        .provider
        .or(config.default_provider.clone())
        .ok_or_else(|| {
            CrabError::ConfigError(
                "No provider specified. Use -p <provider> or set default_provider in config."
                    .to_string(),
            )
        })?;
    let provider = providers::get_provider(&provider_name)?;

    let model = cli
        .model
        .or(config.default_model.clone())
        .ok_or_else(|| {
            CrabError::ConfigError(
                "No model specified. Use -m <model> or set default_model in config.".to_string(),
            )
        })?;

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

/// Handles --list-models. With -a, iterates all providers (errors are
/// silently skipped unless --verbose). Without -a, requires -p or a
/// configured default_provider.
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

/// Returns cached models if valid, otherwise fetches from the provider
/// and updates the cache.
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

    let provider = providers::get_provider(provider_name)?;
    let models = provider.list_models().await?;

    if cache_enabled {
        cache.set(provider_name, models.clone());
    }

    Ok(models)
}
