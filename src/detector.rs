use std::path::{Path, PathBuf};

/// All version constraint files we know how to parse.
const KNOWN_FILES: &[&str] = &[
    ".nvmrc",
    ".node-version",
    ".tool-versions",
    "rust-toolchain.toml",
    "rust-toolchain",
    ".python-version",
    "go.mod",
    "package.json",
];

/// Scan `dir` and return paths to every recognised version-constraint file that exists.
pub fn detect_version_files(dir: &Path) -> Vec<PathBuf> {
    KNOWN_FILES
        .iter()
        .map(|name| dir.join(name))
        .filter(|p| p.is_file())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn detects_existing_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(".nvmrc"), "20").unwrap();
        fs::write(tmp.path().join(".python-version"), "3.11").unwrap();

        let found = detect_version_files(tmp.path());
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn returns_empty_when_no_files() {
        let tmp = TempDir::new().unwrap();
        let found = detect_version_files(tmp.path());
        assert!(found.is_empty());
    }

    #[test]
    fn ignores_unknown_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("random.txt"), "stuff").unwrap();
        let found = detect_version_files(tmp.path());
        assert!(found.is_empty());
    }
}
