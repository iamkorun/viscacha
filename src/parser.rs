use std::fs;
use std::path::Path;

/// A parsed version requirement for a specific tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRequirement {
    pub tool: String,
    pub required: String,
    pub source: String,
}

/// Parse all version requirements from a given constraint file.
/// Returns an empty vec (not an error) for unrecognised or unreadable files.
pub fn parse_version_file(path: &Path) -> Vec<VersionRequirement> {
    let filename = match path.file_name().and_then(|f| f.to_str()) {
        Some(n) => n,
        None => return vec![],
    };

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    match filename {
        ".nvmrc" | ".node-version" => parse_simple_version(&content, "node", filename),
        ".python-version" => parse_simple_version(&content, "python", filename),
        ".tool-versions" => parse_tool_versions(&content),
        "rust-toolchain.toml" => parse_rust_toolchain_toml(&content),
        "rust-toolchain" => parse_simple_version(&content, "rust", filename),
        "go.mod" => parse_go_mod(&content),
        "package.json" => parse_package_json_engines(&content),
        _ => vec![],
    }
}

/// True if `s` looks like a numeric version (starts with a digit).
/// We use this to skip aliases like `lts/iron`, `system`, `stable`, `nightly`,
/// or asdf path/ref forms — viscacha can't compare those against an installed version.
fn is_numeric_version(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_digit())
}

fn parse_simple_version(content: &str, tool: &str, source: &str) -> Vec<VersionRequirement> {
    let trimmed = content.trim().trim_start_matches('v');
    if trimmed.is_empty() || !is_numeric_version(trimmed) {
        return vec![];
    }
    vec![VersionRequirement {
        tool: tool.to_string(),
        required: trimmed.to_string(),
        source: source.to_string(),
    }]
}

fn parse_tool_versions(content: &str) -> Vec<VersionRequirement> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut parts = line.splitn(2, char::is_whitespace);
            let tool_name = parts.next()?.trim();
            let version = parts.next()?.trim().trim_start_matches('v');
            // Skip empty values, asdf placeholders ("system"), and ref/path forms.
            if version.is_empty() || !is_numeric_version(version) {
                return None;
            }
            let mapped_tool = match tool_name {
                "nodejs" | "node" => "node",
                "python" => "python",
                "rust" => "rust",
                "golang" | "go" => "go",
                _ => return None,
            };
            Some(VersionRequirement {
                tool: mapped_tool.to_string(),
                required: version.to_string(),
                source: ".tool-versions".to_string(),
            })
        })
        .collect()
}

fn parse_rust_toolchain_toml(content: &str) -> Vec<VersionRequirement> {
    let table: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let channel = table
        .get("toolchain")
        .and_then(|t| t.get("channel"))
        .and_then(|c| c.as_str());

    // Skip channel aliases like "stable" / "beta" / "nightly" — they're
    // not numeric versions and we can't meaningfully compare them.
    match channel {
        Some(ch) if is_numeric_version(ch) => vec![VersionRequirement {
            tool: "rust".to_string(),
            required: ch.to_string(),
            source: "rust-toolchain.toml".to_string(),
        }],
        _ => vec![],
    }
}

fn parse_go_mod(content: &str) -> Vec<VersionRequirement> {
    for line in content.lines() {
        // Strip end-of-line `// comment` first.
        let line_no_comment = line.split("//").next().unwrap_or(line);
        let trimmed = line_no_comment.trim();
        if let Some(rest) = trimmed.strip_prefix("go ") {
            let version = rest.trim().trim_start_matches('v');
            if !version.is_empty() && is_numeric_version(version) {
                return vec![VersionRequirement {
                    tool: "go".to_string(),
                    required: version.to_string(),
                    source: "go.mod".to_string(),
                }];
            }
        }
    }
    vec![]
}

fn parse_package_json_engines(content: &str) -> Vec<VersionRequirement> {
    let json: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let engines = match json.get("engines") {
        Some(e) => e,
        None => return vec![],
    };

    let mut reqs = Vec::new();

    if let Some(node_req) = engines.get("node").and_then(|v| v.as_str()) {
        reqs.push(VersionRequirement {
            tool: "node".to_string(),
            required: node_req.to_string(),
            source: "package.json".to_string(),
        });
    }

    if let Some(npm_req) = engines.get("npm").and_then(|v| v.as_str()) {
        reqs.push(VersionRequirement {
            tool: "npm".to_string(),
            required: npm_req.to_string(),
            source: "package.json".to_string(),
        });
    }

    reqs
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_and_parse(filename: &str, content: &str) -> Vec<VersionRequirement> {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(filename);
        fs::write(&path, content).unwrap();
        parse_version_file(&path)
    }

    #[test]
    fn parse_nvmrc() {
        let reqs = write_and_parse(".nvmrc", "v20.11.0\n");
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].tool, "node");
        assert_eq!(reqs[0].required, "20.11.0");
    }

    #[test]
    fn parse_nvmrc_major_only() {
        let reqs = write_and_parse(".nvmrc", "20\n");
        assert_eq!(reqs[0].required, "20");
    }

    #[test]
    fn parse_node_version() {
        let reqs = write_and_parse(".node-version", "18.17.0");
        assert_eq!(reqs[0].tool, "node");
        assert_eq!(reqs[0].required, "18.17.0");
    }

    #[test]
    fn parse_python_version() {
        let reqs = write_and_parse(".python-version", "3.11.4\n");
        assert_eq!(reqs[0].tool, "python");
        assert_eq!(reqs[0].required, "3.11.4");
    }

    #[test]
    fn parse_tool_versions_multiple() {
        let content = "nodejs 20.11.0\npython 3.11.4\nrust 1.76.0\ngolang 1.22.0\n";
        let reqs = write_and_parse(".tool-versions", content);
        assert_eq!(reqs.len(), 4);
        assert_eq!(reqs[0].tool, "node");
        assert_eq!(reqs[1].tool, "python");
        assert_eq!(reqs[2].tool, "rust");
        assert_eq!(reqs[3].tool, "go");
    }

    #[test]
    fn parse_tool_versions_skips_unknown() {
        let content = "ruby 3.2.0\nnodejs 20.0.0\n";
        let reqs = write_and_parse(".tool-versions", content);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].tool, "node");
    }

    #[test]
    fn parse_tool_versions_ignores_comments() {
        let content = "# comment\nnodejs 20.0.0\n";
        let reqs = write_and_parse(".tool-versions", content);
        assert_eq!(reqs.len(), 1);
    }

    #[test]
    fn parse_rust_toolchain_toml() {
        let content = "[toolchain]\nchannel = \"1.76.0\"\n";
        let reqs = write_and_parse("rust-toolchain.toml", content);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].tool, "rust");
        assert_eq!(reqs[0].required, "1.76.0");
    }

    #[test]
    fn parse_rust_toolchain_plain() {
        let reqs = write_and_parse("rust-toolchain", "1.76.0\n");
        assert_eq!(reqs[0].tool, "rust");
        assert_eq!(reqs[0].required, "1.76.0");
    }

    #[test]
    fn parse_go_mod() {
        let content = "module example.com/foo\n\ngo 1.22.0\n";
        let reqs = write_and_parse("go.mod", content);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].tool, "go");
        assert_eq!(reqs[0].required, "1.22.0");
    }

    #[test]
    fn parse_package_json_engines() {
        let content = r#"{"name":"foo","engines":{"node":">=18","npm":">=9"}}"#;
        let reqs = write_and_parse("package.json", content);
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0].tool, "node");
        assert_eq!(reqs[0].required, ">=18");
        assert_eq!(reqs[1].tool, "npm");
    }

    #[test]
    fn parse_package_json_no_engines() {
        let content = r#"{"name":"foo","version":"1.0.0"}"#;
        let reqs = write_and_parse("package.json", content);
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_empty_file() {
        let reqs = write_and_parse(".nvmrc", "");
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_nonexistent_file() {
        let reqs = parse_version_file(Path::new("/tmp/does-not-exist/.nvmrc"));
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_nvmrc_only_v_prefix() {
        let reqs = write_and_parse(".nvmrc", "v\n");
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_tool_versions_go_alias() {
        let content = "go 1.22.0\n";
        let reqs = write_and_parse(".tool-versions", content);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].tool, "go");
    }

    #[test]
    fn parse_tool_versions_empty() {
        let reqs = write_and_parse(".tool-versions", "");
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_tool_versions_only_comments() {
        let reqs = write_and_parse(".tool-versions", "# just a comment\n# another\n");
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_malformed_toml() {
        let reqs = write_and_parse("rust-toolchain.toml", "not valid toml {{{");
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_malformed_json() {
        let reqs = write_and_parse("package.json", "{invalid json}");
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_rust_toolchain_toml_no_channel() {
        let content = "[toolchain]\ncomponents = [\"clippy\"]\n";
        let reqs = write_and_parse("rust-toolchain.toml", content);
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_go_mod_no_go_directive() {
        let content = "module example.com/foo\n\nrequire golang.org/x/text v0.14.0\n";
        let reqs = write_and_parse("go.mod", content);
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_go_mod_with_inline_comment() {
        let content = "module example.com/foo\n\ngo 1.22.0 // toolchain pinned\n";
        let reqs = write_and_parse("go.mod", content);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].required, "1.22.0");
    }

    #[test]
    fn parse_nvmrc_skips_lts_alias() {
        let reqs = write_and_parse(".nvmrc", "lts/iron\n");
        assert!(reqs.is_empty(), "lts aliases should be skipped");
    }

    #[test]
    fn parse_nvmrc_skips_node_alias() {
        let reqs = write_and_parse(".nvmrc", "latest\n");
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_python_version_skips_pyenv_aliases() {
        // pyenv supports things like "system" and dev tags
        let reqs = write_and_parse(".python-version", "system\n");
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_tool_versions_skips_system() {
        let content = "nodejs system\npython 3.11.4\n";
        let reqs = write_and_parse(".tool-versions", content);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].tool, "python");
    }

    #[test]
    fn parse_tool_versions_skips_path_form() {
        let content = "nodejs path:/usr/local/bin/node\nrust 1.76.0\n";
        let reqs = write_and_parse(".tool-versions", content);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].tool, "rust");
    }

    #[test]
    fn parse_rust_toolchain_toml_skips_stable_channel() {
        let content = "[toolchain]\nchannel = \"stable\"\n";
        let reqs = write_and_parse("rust-toolchain.toml", content);
        assert!(reqs.is_empty(), "stable channel should be skipped");
    }

    #[test]
    fn parse_rust_toolchain_toml_skips_nightly_dated() {
        // Dated nightlies like "nightly-2024-01-15" still aren't comparable
        let content = "[toolchain]\nchannel = \"nightly-2024-01-15\"\n";
        let reqs = write_and_parse("rust-toolchain.toml", content);
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_rust_toolchain_plain_skips_stable() {
        let reqs = write_and_parse("rust-toolchain", "stable\n");
        assert!(reqs.is_empty());
    }

    #[test]
    fn parse_tool_versions_handles_tab_separator() {
        let content = "nodejs\t20.11.0\n";
        let reqs = write_and_parse(".tool-versions", content);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].required, "20.11.0");
    }

    #[test]
    fn parse_package_json_engines_with_compound_range() {
        let content = r#"{"engines":{"node":">=18 <22"}}"#;
        let reqs = write_and_parse("package.json", content);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].required, ">=18 <22");
    }

    #[test]
    fn parse_package_json_engines_with_caret() {
        let content = r#"{"engines":{"node":"^18.0.0"}}"#;
        let reqs = write_and_parse("package.json", content);
        assert_eq!(reqs[0].required, "^18.0.0");
    }
}
