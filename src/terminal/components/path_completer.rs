use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct PathCompleter {
    last_query: String,
    candidates: Vec<String>,
    candidate_index: usize,
}

impl PathCompleter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset any cached candidates
    pub fn reset(&mut self) {
        self.last_query.clear();
        self.candidates.clear();
        self.candidate_index = 0;
    }

    /// Resolve ~ to user's home directory
    fn expand_tilde(path_str: &str) -> String {
        if path_str == "~" || path_str.starts_with("~/") || path_str.starts_with("~\\") {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".into());
            if path_str == "~" {
                home
            } else {
                format!("{}{}", home, &path_str[1..])
            }
        } else {
            path_str.to_string()
        }
    }

    /// Complete the given path query.
    /// If query matches last_query and multiple candidates exist, cycles to next candidate.
    /// Returns the completed path string.
    pub fn complete(&mut self, current_input: &str) -> Option<String> {
        if current_input.is_empty() {
            return None;
        }

        // Check if cycling through existing candidates
        if !self.candidates.is_empty() && self.last_query == current_input {
            self.candidate_index = (self.candidate_index + 1) % self.candidates.len();
            let result = self.candidates[self.candidate_index].clone();
            self.last_query = result.clone();
            return Some(result);
        }

        // New completion search
        let expanded = Self::expand_tilde(current_input);
        let path = Path::new(&expanded);

        let (parent_dir, prefix, original_prefix) = if current_input.ends_with('/') || current_input.ends_with('\\') {
            (path.to_path_buf(), "".to_string(), current_input.to_string())
        } else {
            let parent = path.parent().unwrap_or_else(|| Path::new(""));
            let file_stem = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let orig_parent = if let Some(idx) = current_input.rfind(['/', '\\']) {
                &current_input[..=idx]
            } else {
                ""
            };
            (
                if parent.as_os_str().is_empty() {
                    PathBuf::from(".")
                } else {
                    parent.to_path_buf()
                },
                file_stem.to_string(),
                orig_parent.to_string(),
            )
        };

        let mut matches = Vec::new();
        if let Ok(entries) = fs::read_dir(&parent_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&prefix) {
                    let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                    let suffix = if is_dir { "/" } else { "" };
                    matches.push(format!("{}{}{}", original_prefix, name, suffix));
                }
            }
        }

        matches.sort();

        if matches.is_empty() {
            self.reset();
            None
        } else {
            self.candidates = matches;
            self.candidate_index = 0;
            let result = self.candidates[0].clone();
            self.last_query = result.clone();
            Some(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_tilde_correctly() {
        let expanded = PathCompleter::expand_tilde("~/test");
        assert!(!expanded.starts_with('~'));
    }

    #[test]
    fn complete_empty_returns_none() {
        let mut completer = PathCompleter::new();
        assert_eq!(completer.complete(""), None);
    }
}
