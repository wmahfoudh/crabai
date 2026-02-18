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
async fn run(cli: Cli) -> Result<(), CrabError> {
    // Interactive config mode: launch wizard and exit
    if cli.config {
        return config_editor::run_interactive_config(cli.use_config.as_deref()).await;
    }

    let config = Config::load(cli.use_config.as_deref())?;

    // Auto-install bundled prompts if any are missing
    let prompts_dir = config.prompts_dir();
    if BundledPrompts::has_missing_prompts(&prompts_dir) {
        let _ = BundledPrompts::install_to(&prompts_dir);
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
            eprintln!("No model configured.");
            eprintln!("Would you like to create a configuration file now? (y/n)");

            let mut response = String::new();
            std::io::stdin().read_line(&mut response)?;

            if response.trim().to_lowercase().starts_with('y') {
                config_editor::run_interactive_config(cli.use_config.as_deref()).await?;
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
            Ok(content) => (content, &cli.args[1..]),
            Err(_) => (first_arg.to_string(), &cli.args[1..]),
        }
    } else {
        ("".to_string(), &[][..])
    };

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

    let config_dir = Config::config_dir();
    let mut cache = ModelCache::load(&config_dir);
    let ttl = config.cache_ttl_hours();
    let cache_enabled = config.model_cache_enabled();

    // Resolve model capabilities from cache to determine token limits and parameter support.
    let model_info = if cache_enabled {
        get_models(&provider_name, &mut cache, ttl, true, &config)
            .await
            .ok()
            .and_then(|models| models.into_iter().find(|m| m.id == model_name))
    } else {
        None
    };

    let temperature = cli
        .temperature
        .unwrap_or_else(|| config.resolve_temperature());

    let max_tokens = match cli.max_tokens {
        Some(s) => {
            if s.to_lowercase() == "max" {
                model_info
                    .as_ref()
                    .and_then(|m| m.max_output_tokens)
                    .unwrap_or(4096)
            } else {
                s.parse::<u32>()
                    .unwrap_or_else(|_| config.resolve_max_tokens())
            }
        }
        None => config.resolve_max_tokens(),
    };

    let (mut final_temperature, mut final_max_tokens) =
        provider.sanitize_params(&model_name, temperature, max_tokens);

    // Apply cached info if it exists and contradicts/enriches
    if let Some(info) = &model_info {
        if !info.supports_temperature {
            final_temperature = None;
        }
        if let Some(limit) = info.max_output_tokens {
            final_max_tokens = final_max_tokens.min(limit);
        }
    }

    if cli.verbose {
        eprintln!("Provider: {}", provider.name());
        eprintln!("Model: {model_name}");
        eprintln!("Temperature (requested): {temperature}");
        eprintln!("Temperature (final): {:?}", final_temperature);
        eprintln!("Max tokens (requested): {max_tokens}");
        eprintln!("Max tokens (final): {final_max_tokens}");
    }

    let response_result = provider
        .send(
            &model_name,
            &final_prompt,
            final_temperature,
            final_max_tokens,
            model_info.as_ref().and_then(|m| m.max_tokens_param.clone()),
        )
        .await;

    match response_result {
        Ok(response) => {
            print!("{response}");
            Ok(())
        }
        Err(e) => {
            // Extract model constraints from provider error messages to update local cache.
            if let CrabError::ProviderError { message, .. } = &e {
                let mut updated = false;
                let mut info = model_info.unwrap_or_else(|| types::ModelInfo::new(&model_name));

                if let Some(new_limit) = try_extract_limit(message) {
                    if cli.verbose {
                        eprintln!("Learning: Detected new token limit for {model_name}: {new_limit}");
                    }
                    info.max_output_tokens = Some(new_limit);
                    updated = true;
                }

                if let Some(new_param) = try_extract_param_name(message) {
                    if cli.verbose {
                        eprintln!("Learning: Detected new token parameter name for {model_name}: {new_param}");
                    }
                    info.max_tokens_param = Some(new_param);
                    updated = true;
                }

                if updated {
                    cache.update_model(&provider_name, info);
                    let _ = cache.save(&config_dir);
                }
            }
            Err(e)
        }
    }
}

fn try_extract_limit(message: &str) -> Option<u32> {
    // Case 1: Anthropic "64000 > 8192"
    if let Some(caps) = regex::Regex::new(r"(\d+)\s*>\s*(\d+)").ok().and_then(|re| re.captures(message)) {
        return caps.get(2).and_then(|m| m.as_str().parse().ok());
    }

    // Case 2: OpenAI or others "limit of 4096"
    if let Some(caps) = regex::Regex::new(r"(?i)limit of (\d+)").ok().and_then(|re| re.captures(message)) {
        return caps.get(1).and_then(|m| m.as_str().parse().ok());
    }

    // Case 3: "maximum allowed is 8192"
    if let Some(caps) = regex::Regex::new(r"(?i)maximum(?: allowed)? (?:is )?(\d+)").ok().and_then(|re| re.captures(message)) {
        return caps.get(1).and_then(|m| m.as_str().parse().ok());
    }

    None
}

fn try_extract_param_name(message: &str) -> Option<String> {
    // Case: "Use 'max_completion_tokens' instead."
    let re = regex::Regex::new(r"(?i)use\s+'([^']+)'\s+instead").ok()?;
    re.captures(message)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

/// Handles the --list-models command.
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
                    all_models.push(format!("{name}:{}", m.id));
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
) -> Result<Vec<types::ModelInfo>, CrabError> {
    if cache_enabled {
        if let Some(cached) = cache.get(provider_name, ttl) {
            return Ok(cached);
        }
    }

    let provider = get_provider_with_config(provider_name, config)?;
    let models = provider.list_models().await?;

    if cache_enabled {
        cache.set(provider_name, models.clone());
    }

    Ok(models)
}
