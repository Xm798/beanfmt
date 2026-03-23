use crate::line::{parse_line, Line};
use crate::options::Options;
use glob::glob;
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

pub struct FormattedFile {
    pub path: PathBuf,
    pub content: String,
}

/// Extract include paths from beancount content, resolving them relative to base_dir.
/// Glob patterns in include paths are expanded.
pub fn extract_includes(content: &str, base_dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for line in content.lines() {
        if let Line::Include { path } = parse_line(line) {
            let resolved = base_dir.join(path);
            let pattern = resolved.to_string_lossy();
            match glob(&pattern) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        paths.push(entry);
                    }
                }
                Err(_) => {
                    // If glob pattern is invalid, try as literal path
                    if resolved.exists() {
                        paths.push(resolved);
                    }
                }
            }
        }
    }
    paths
}

/// Format a beancount file and all its includes using BFS traversal.
/// Returns a list of formatted files in BFS visit order.
/// Cycles are detected and broken. Missing files are skipped.
pub fn format_recursive(root_path: &Path, options: &Options) -> Vec<FormattedFile> {
    let mut results = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    let root = match fs::canonicalize(root_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("warning: cannot resolve {}: {e}", root_path.display());
            return results;
        }
    };

    queue.push_back(root.clone());
    visited.insert(root);

    while let Some(path) = queue.pop_front() {
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("warning: skipping {}: {e}", path.display());
                continue;
            }
        };

        let formatted = crate::format(&raw, options);

        let base_dir = path.parent().unwrap_or(Path::new("."));
        let includes = extract_includes(&raw, base_dir);

        for inc in includes {
            if let Ok(canonical) = fs::canonicalize(&inc) && visited.insert(canonical.clone()) {
                queue.push_back(canonical);
            }
        }

        results.push(FormattedFile {
            path,
            content: formatted,
        });
    }

    results
}
