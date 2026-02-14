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

/// Combine prompt template with optional STDIN content. When STDIN is
/// present, the two are joined with a "\n\n-----\n\n" separator.
pub fn assemble(prompt: &str, stdin: Option<&str>) -> String {
    match stdin {
        Some(input) => format!("{prompt}\n\n-----\n\n{input}"),
        None => prompt.to_string(),
    }
}
