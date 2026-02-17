use std::path::Path;

use crate::error::CrabError;

/// Load prompt content from {prompts_dir}/{name}.md.
pub fn load_prompt(name: &str, prompts_dir: &Path) -> Result<String, CrabError> {
    let path = prompts_dir.join(format!("{name}.md"));
    if !path.exists() {
        return Err(CrabError::PromptNotFound(name.to_string()));
    }
    Ok(std::fs::read_to_string(&path)?)
}

/// List available prompt names (filenames without .md extension), sorted.
/// Returns an empty list if the prompts directory does not exist.
pub fn list_prompts(prompts_dir: &Path) -> Result<Vec<String>, CrabError> {
    if !prompts_dir.exists() {
        return Ok(Vec::new());
    }

    let mut prompts = Vec::new();
    for entry in std::fs::read_dir(prompts_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                prompts.push(stem.to_string());
            }
        }
    }
    prompts.sort();
    Ok(prompts)
}

/// Combine prompt template, additional arguments, and optional STDIN content.
/// All parts are joined with a consistent separator.
pub fn assemble(prompt: &str, args: &[String], stdin: Option<&str>) -> String {
    let mut final_prompt = String::from(prompt);

    for arg in args {
        final_prompt.push_str("\n\n");
        final_prompt.push_str(arg);
    }

    if let Some(input) = stdin {
        final_prompt.push_str("\n\n");
        final_prompt.push_str(input);
    }

    final_prompt
}
