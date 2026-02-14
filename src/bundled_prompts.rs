use std::path::PathBuf;
use include_dir::{include_dir, Dir};
use crate::error::CrabError;

/// Bundled prompts directory embedded in the binary at compile time.
/// Automatically includes ALL .md files from src/prompts/.
static PROMPTS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/prompts");

/// Bundled prompts that ship with CrabAI.
/// All .md files in src/prompts/ are automatically included.
pub struct BundledPrompts;

impl BundledPrompts {
    /// Install bundled prompts to the specified directory.
    /// Only creates files that don't already exist (never overwrites).
    pub fn install_to(dir: &PathBuf) -> Result<usize, CrabError> {
        std::fs::create_dir_all(dir)?;
        
        let mut installed = 0;
        
        // Iterate through all files in the embedded directory
        for file in PROMPTS_DIR.files() {
            if let Some(filename) = file.path().file_name() {
                let path = dir.join(filename);
                
                // Only install if file doesn't exist
                if !path.exists() {
                    if let Some(contents) = file.contents_utf8() {
                        std::fs::write(&path, contents)?;
                        installed += 1;
                    }
                }
            }
        }
        
        Ok(installed)
    }

    /// Check if any bundled prompts are missing from the directory.
    pub fn has_missing_prompts(dir: &PathBuf) -> bool {
        if !dir.exists() {
            return true;
        }
        
        // Check if any embedded file is missing from the target directory
        PROMPTS_DIR.files().any(|file| {
            if let Some(filename) = file.path().file_name() {
                !dir.join(filename).exists()
            } else {
                false
            }
        })
    }
}
