use glob::Pattern as GlobPattern;
use std::fs;
use std::path::{Path, PathBuf};

pub struct GitIgnore {
    ignore_patterns: Vec<Pattern>,
}

pub struct Pattern {
    pattern: GlobPattern,
    original_pattern: String,
}

impl Pattern {
    pub fn new(pattern: String) -> Self {
        // TODO: Implement a more complete version of gitignore.
        let glob_pattern = match pattern.starts_with("/") {
            true => GlobPattern::new(&pattern).expect("Failed to compile glob pattern"),
            false => GlobPattern::new(&format!("**/{}", pattern))
                .expect("Failed to compile glob pattern"),
        };
        Pattern {
            pattern: glob_pattern,
            original_pattern: pattern,
        }
    }

    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path
            .to_str()
            .expect("Path could not be converted to string");
        path_str.contains(&self.original_pattern) || self.pattern.matches(path_str)
    }
}

impl GitIgnore {
    pub fn new(start_path: &Path) -> Self {
        let mut current_path = start_path.to_path_buf();
        let mut gitignore_path: PathBuf;

        // Walk up the directory tree to find the .gitignore file
        loop {
            gitignore_path = current_path.join(".gitignore");
            if gitignore_path.exists() {
                break;
            }
            if !current_path.pop() {
                panic!("No .gitignore file found in any parent directory");
            }
        }

        // Read the .gitignore file
        let gitignore_content =
            fs::read_to_string(gitignore_path).expect("Failed to read .gitignore file");

        // Split the content into lines, create Pattern instances, and collect them into a Vec<Pattern>
        let ignore_patterns = gitignore_content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with("#")) // Filter out empty lines and comments
            .map(|pattern| Pattern::new(pattern))
            .collect::<Vec<_>>();
        GitIgnore { ignore_patterns }
    }

    pub fn ignores(&self, path: &Path) -> bool {
        self.ignore_patterns
            .iter()
            .any(|pattern| pattern.matches(path))
    }
}
