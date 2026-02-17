use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::PathBuf;

use crate::config::Config;
use crate::error::CrabError;
use crate::providers::{self, get_provider_with_config};

/// Interactive configuration wizard for creating or editing CrabAI config files.
///
/// Workflow:
/// 1. Load existing config if present, otherwise start with defaults
/// 2. Prompt for default provider (from list of supported providers)
/// 3. Fetch and display models for selected provider
/// 4. Prompt for default model (from fetched list or manual entry)
/// 5. Prompt for temperature, max_tokens, prompts_dir, and cache settings
/// 6. Optionally configure advanced settings (custom API key env vars)
/// 7. Save the updated config to disk
///
/// All prompts show current values as defaults for easy editing.
pub async fn run_interactive_config(config_path: Option<&str>) -> Result<(), CrabError> {
    let theme = ColorfulTheme::default();
    let path = match config_path {
        Some(p) => PathBuf::from(p),
        None => Config::default_config_path(),
    };

    println!("CrabAI Interactive Configuration");
    println!("=================================\n");

    let mut config = if path.exists() {
        println!("Loading existing config from: {}", path.display());
        Config::load(Some(path.to_str().unwrap()))?
    } else {
        println!("Creating new config at: {}", path.display());
        Config::default()
    };

    // Provider and model selection loop
    loop {
        let provider_names = providers::list_provider_names();
        let default_provider_name = config
            .default_provider
            .as_deref()
            .unwrap_or(provider_names[0]);
        let default_idx = provider_names
            .iter()
            .position(|&n| n == default_provider_name)
            .unwrap_or(0);

        let provider_idx = Select::with_theme(&theme)
            .with_prompt("Select default provider")
            .items(&provider_names)
            .default(default_idx)
            .interact()?;

        let selected_provider = provider_names[provider_idx];
        config.default_provider = Some(selected_provider.to_string());

        println!("\nFetching models for {}...", selected_provider);
        let provider = get_provider_with_config(selected_provider, &config)?;
        let mut models = match provider.list_models().await {
            Ok(models) => models,
            Err(e) => {
                eprintln!("Warning: Could not fetch models: {e}");
                vec![]
            }
        };

        // Add a "Go Back" option
        models.insert(0, "<-- Go Back".to_string());

        let model_idx = Select::with_theme(&theme)
            .with_prompt("Select default model")
            .items(&models)
            .default(0)
            .interact()?;

        if model_idx == 0 {
            // User selected "Go Back"
            println!(); // Add a newline for better formatting
            continue;
        }

        let selected_model = models[model_idx].clone();
        config.default_model = Some(format!("{}:{}", selected_provider, selected_model));
        break;
    }

    // Temperature
    let temp_str = config
        .temperature
        .map(|t| t.to_string())
        .unwrap_or_else(|| "0.2".to_string());

    let temperature: String = Input::with_theme(&theme)
        .with_prompt("Temperature (0.0 to 2.0)")
        .default(temp_str)
        .interact_text()?;

    config.temperature = Some(temperature.parse().unwrap_or(0.2));

    // Max tokens
    let use_max_tokens = Confirm::with_theme(&theme)
        .with_prompt("Set max tokens limit? (choose No for provider default)")
        .default(config.max_tokens.is_some())
        .interact()?;

    if use_max_tokens {
        let max_tokens_str = config
            .max_tokens
            .map(|t| t.to_string())
            .unwrap_or_else(|| "4096".to_string());

        let max_tokens: String = Input::with_theme(&theme)
            .with_prompt("Max tokens")
            .default(max_tokens_str)
            .interact_text()?;

        config.max_tokens = Some(max_tokens.parse().unwrap_or(4096));
    } else {
        config.max_tokens = None;
    }

    // Prompts directory
    let default_prompts_dir = config
        .prompts_dir
        .clone()
        .unwrap_or_else(|| "~/.config/crabai/prompts".to_string());

    let prompts_dir: String = Input::with_theme(&theme)
        .with_prompt("Prompts directory")
        .default(default_prompts_dir)
        .interact_text()?;

    config.prompts_dir = Some(prompts_dir);

    // Model cache settings
    let cache_enabled = Confirm::with_theme(&theme)
        .with_prompt("Enable model list caching?")
        .default(config.model_cache_enabled())
        .interact()?;

    config.model_cache = Some(cache_enabled);

    if cache_enabled {
        let ttl_str = config.model_cache_ttl_hours.unwrap_or(24).to_string();

        let ttl: String = Input::with_theme(&theme)
            .with_prompt("Cache TTL (hours)")
            .default(ttl_str)
            .interact_text()?;

        config.model_cache_ttl_hours = Some(ttl.parse().unwrap_or(24));
    }

    // Advanced configuration - always save defaults
    let configure_advanced = Confirm::with_theme(&theme)
        .with_prompt("Configure advanced settings (API key environment variables)?")
        .default(false)
        .interact()?;

    // Initialize advanced config with defaults for all providers
    let mut advanced = config.advanced.clone().unwrap_or_default();
    let mut api_key_vars = advanced.api_key_vars.clone().unwrap_or_default();

    if configure_advanced {
        // Interactive mode: prompt user for each provider
        println!("\nAdvanced Configuration");
        println!("----------------------");
        println!("Customize environment variable names for API keys.");
        println!("Press Enter to keep the default.\n");
    }

    // Ensure all providers have their default API key vars in the map
    for provider_name in providers::list_provider_names() {
        let default_var = Config::default_api_key_var(provider_name);

        if configure_advanced {
            let current_var = api_key_vars
                .get(provider_name)
                .cloned()
                .unwrap_or_else(|| default_var.clone());

            let var_name: String = Input::with_theme(&theme)
                .with_prompt(format!("{} API key env var", provider_name))
                .default(current_var)
                .interact_text()?;

            api_key_vars.insert(provider_name.to_string(), var_name);
        } else {
            // Not configuring: ensure default is in the map if not already present
            if !api_key_vars.contains_key(provider_name) {
                api_key_vars.insert(provider_name.to_string(), default_var);
            }
        }
    }

    // Always save the api_key_vars map (with defaults)
    advanced.api_key_vars = Some(api_key_vars);
    config.advanced = Some(advanced);

    // Save configuration
    println!("\nSaving configuration to: {}", path.display());
    config.save(&path)?;
    println!("✓ Configuration saved successfully!");

    Ok(())
}
