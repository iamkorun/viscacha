use std::process::Command;

use crate::parser::VersionRequirement;

/// Result of checking one tool's installed version against a requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub tool: String,
    pub required: String,
    pub installed: Option<String>,
    pub status: CheckStatus,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Fail,
    NotInstalled,
    ParseError(String),
}

impl CheckResult {
    /// Return the fix command for a failing check, if applicable.
    pub fn fix_command(&self) -> Option<String> {
        match self.status {
            CheckStatus::Fail | CheckStatus::NotInstalled => {}
            _ => return None,
        }

        let req = &self.required;
        match self.tool.as_str() {
            "node" => Some(format!("nvm install {req} && nvm use {req}")),
            "python" => Some(format!("pyenv install {req} && pyenv local {req}")),
            "rust" => Some(format!("rustup toolchain install {req} && rustup override set {req}")),
            "go" => Some(format!("go install golang.org/dl/go{req}@latest && go{req} download")),
            _ => None,
        }
    }
}

/// Check a single version requirement against the installed version.
pub fn check_requirement(req: &VersionRequirement) -> CheckResult {
    let installed = get_installed_version(&req.tool);

    let (installed_str, status) = match installed {
        None => (None, CheckStatus::NotInstalled),
        Some(ver) => {
            let matches = version_matches(&req.required, &ver);
            (Some(ver), if matches { CheckStatus::Pass } else { CheckStatus::Fail })
        }
    };

    CheckResult {
        tool: req.tool.clone(),
        required: req.required.clone(),
        installed: installed_str,
        status,
        source: req.source.clone(),
    }
}

/// Check all requirements and return results.
pub fn check_all(reqs: &[VersionRequirement]) -> Vec<CheckResult> {
    reqs.iter().map(check_requirement).collect()
}

/// Get the installed version of a tool by running its version command.
fn get_installed_version(tool: &str) -> Option<String> {
    let (cmd, args) = match tool {
        "node" => ("node", vec!["--version"]),
        "python" => ("python3", vec!["--version"]),
        "rust" => ("rustc", vec!["--version"]),
        "go" => ("go", vec!["version"]),
        "npm" => ("npm", vec!["--version"]),
        _ => return None,
    };

    let output = Command::new(cmd).args(&args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    extract_version_from_output(tool, &stdout)
}

/// Extract a clean version string from the command output.
fn extract_version_from_output(tool: &str, output: &str) -> Option<String> {
    let trimmed = output.trim();
    match tool {
        "node" => Some(trimmed.trim_start_matches('v').to_string()),
        "python" => {
            // "Python 3.11.4"
            trimmed.strip_prefix("Python ").map(|v| v.to_string())
        }
        "rust" => {
            // "rustc 1.76.0 (07dca489a 2024-02-04)"
            trimmed
                .strip_prefix("rustc ")
                .and_then(|rest| rest.split_whitespace().next())
                .map(|v| v.to_string())
        }
        "go" => {
            // "go version go1.22.0 linux/amd64"
            trimmed
                .split_whitespace()
                .find(|w| w.starts_with("go1") || w.starts_with("go0"))
                .map(|w| w.trim_start_matches("go").to_string())
        }
        "npm" => Some(trimmed.to_string()),
        _ => Some(trimmed.to_string()),
    }
}

/// Check if an installed version satisfies a requirement string.
fn version_matches(required: &str, installed: &str) -> bool {
    let req = required.trim();
    let inst = installed.trim();

    // Handle OR ranges first
    if req.contains("||") {
        return req.split("||").any(|part| version_matches(part.trim(), inst));
    }
    // Handle ">=X <Y" style compound ranges (space separated, multiple constraints)
    let parts: Vec<&str> = req.split_whitespace().collect();
    if parts.len() > 1 {
        return parts.iter().all(|p| version_matches(p, inst));
    }

    // Handle range operators from package.json engines
    if req.starts_with(">=") {
        return match_gte(req.trim_start_matches(">=").trim(), inst);
    }
    if req.starts_with("<=") {
        return match_lte(req.trim_start_matches("<=").trim(), inst);
    }
    if req.starts_with('>') && !req.starts_with(">=") {
        return match_gt(req.trim_start_matches('>').trim(), inst);
    }
    if req.starts_with('<') && !req.starts_with("<=") {
        return match_lt(req.trim_start_matches('<').trim(), inst);
    }
    if req.starts_with('~') {
        return match_tilde(req.trim_start_matches('~').trim(), inst);
    }
    if req.starts_with('^') {
        return match_caret(req.trim_start_matches('^').trim(), inst);
    }

    // Exact or prefix match
    if req.ends_with(".x") || req.ends_with(".*") {
        let prefix = &req[..req.len() - 2];
        return inst.starts_with(prefix);
    }

    // Simple version: could be major-only ("20"), major.minor ("3.11"), or full ("1.76.0")
    let req_parts = parse_version_parts(req);
    let inst_parts = parse_version_parts(inst);

    // Match only the parts that are specified in the requirement
    for (r, i) in req_parts.iter().zip(inst_parts.iter()) {
        if r != i {
            return false;
        }
    }
    true
}

fn parse_version_parts(v: &str) -> Vec<u64> {
    v.split('.')
        .filter_map(|p| {
            // Handle pre-release suffixes like "1.76.0-nightly"
            let numeric = p.split('-').next().unwrap_or(p);
            numeric.parse::<u64>().ok()
        })
        .collect()
}

fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let pa = parse_version_parts(a);
    let pb = parse_version_parts(b);
    let max_len = pa.len().max(pb.len());
    for i in 0..max_len {
        let va = pa.get(i).copied().unwrap_or(0);
        let vb = pb.get(i).copied().unwrap_or(0);
        match va.cmp(&vb) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

fn match_gte(req: &str, installed: &str) -> bool {
    matches!(
        compare_versions(installed, req),
        std::cmp::Ordering::Greater | std::cmp::Ordering::Equal
    )
}

fn match_lte(req: &str, installed: &str) -> bool {
    matches!(
        compare_versions(installed, req),
        std::cmp::Ordering::Less | std::cmp::Ordering::Equal
    )
}

fn match_gt(req: &str, installed: &str) -> bool {
    compare_versions(installed, req) == std::cmp::Ordering::Greater
}

fn match_lt(req: &str, installed: &str) -> bool {
    compare_versions(installed, req) == std::cmp::Ordering::Less
}

fn match_tilde(req: &str, installed: &str) -> bool {
    // ~1.2.3 means >=1.2.3 <1.3.0 (patch-level changes)
    let req_parts = parse_version_parts(req);
    let inst_parts = parse_version_parts(installed);

    if req_parts.len() < 2 || inst_parts.len() < 2 {
        return version_matches(req, installed);
    }

    // Major and minor must match
    req_parts[0] == inst_parts[0]
        && req_parts[1] == inst_parts[1]
        && match_gte(req, installed)
}

fn match_caret(req: &str, installed: &str) -> bool {
    // ^1.2.3 means >=1.2.3 <2.0.0 (minor/patch changes OK)
    let req_parts = parse_version_parts(req);
    let inst_parts = parse_version_parts(installed);

    if req_parts.is_empty() || inst_parts.is_empty() {
        return false;
    }

    // Major must match
    req_parts[0] == inst_parts[0] && match_gte(req, installed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_exact() {
        assert!(version_matches("1.76.0", "1.76.0"));
        assert!(!version_matches("1.76.0", "1.77.0"));
    }

    #[test]
    fn version_matches_major_only() {
        assert!(version_matches("20", "20.11.0"));
        assert!(version_matches("20", "20.0.0"));
        assert!(!version_matches("20", "18.17.0"));
    }

    #[test]
    fn version_matches_major_minor() {
        assert!(version_matches("3.11", "3.11.4"));
        assert!(!version_matches("3.11", "3.12.0"));
    }

    #[test]
    fn version_matches_gte() {
        assert!(version_matches(">=18", "20.11.0"));
        assert!(version_matches(">=18", "18.0.0"));
        assert!(!version_matches(">=18", "16.0.0"));
    }

    #[test]
    fn version_matches_lte() {
        assert!(version_matches("<=20", "18.0.0"));
        assert!(version_matches("<=20", "20.0.0"));
        assert!(!version_matches("<=20", "22.0.0"));
    }

    #[test]
    fn version_matches_gt() {
        assert!(version_matches(">18", "20.0.0"));
        assert!(!version_matches(">18", "18.0.0"));
    }

    #[test]
    fn version_matches_lt() {
        assert!(version_matches("<20", "18.0.0"));
        assert!(!version_matches("<20", "20.0.0"));
    }

    #[test]
    fn version_matches_tilde() {
        assert!(version_matches("~1.2.3", "1.2.5"));
        assert!(version_matches("~1.2.3", "1.2.3"));
        assert!(!version_matches("~1.2.3", "1.3.0"));
    }

    #[test]
    fn version_matches_caret() {
        assert!(version_matches("^1.2.3", "1.5.0"));
        assert!(version_matches("^1.2.3", "1.2.3"));
        assert!(!version_matches("^1.2.3", "2.0.0"));
        assert!(!version_matches("^1.2.3", "1.1.0"));
    }

    #[test]
    fn version_matches_or() {
        assert!(version_matches(">=16 || >=18", "20.0.0"));
        assert!(version_matches("14 || 16 || 18", "18.0.0"));
        assert!(!version_matches("14 || 16", "18.0.0"));
    }

    #[test]
    fn version_matches_wildcard() {
        assert!(version_matches("20.x", "20.11.0"));
        assert!(!version_matches("20.x", "18.0.0"));
    }

    #[test]
    fn extract_node_version() {
        assert_eq!(
            extract_version_from_output("node", "v20.11.0\n"),
            Some("20.11.0".to_string())
        );
    }

    #[test]
    fn extract_python_version() {
        assert_eq!(
            extract_version_from_output("python", "Python 3.11.4\n"),
            Some("3.11.4".to_string())
        );
    }

    #[test]
    fn extract_rust_version() {
        assert_eq!(
            extract_version_from_output("rust", "rustc 1.76.0 (07dca489a 2024-02-04)\n"),
            Some("1.76.0".to_string())
        );
    }

    #[test]
    fn extract_go_version() {
        assert_eq!(
            extract_version_from_output("go", "go version go1.22.0 linux/amd64\n"),
            Some("1.22.0".to_string())
        );
    }

    #[test]
    fn extract_npm_version() {
        assert_eq!(
            extract_version_from_output("npm", "10.2.4\n"),
            Some("10.2.4".to_string())
        );
    }

    #[test]
    fn fix_command_node() {
        let r = CheckResult {
            tool: "node".to_string(),
            required: "20".to_string(),
            installed: Some("18.0.0".to_string()),
            status: CheckStatus::Fail,
            source: ".nvmrc".to_string(),
        };
        assert_eq!(r.fix_command(), Some("nvm install 20 && nvm use 20".to_string()));
    }

    #[test]
    fn fix_command_not_needed_when_pass() {
        let r = CheckResult {
            tool: "node".to_string(),
            required: "20".to_string(),
            installed: Some("20.0.0".to_string()),
            status: CheckStatus::Pass,
            source: ".nvmrc".to_string(),
        };
        assert!(r.fix_command().is_none());
    }

    #[test]
    fn fix_command_for_not_installed() {
        let r = CheckResult {
            tool: "rust".to_string(),
            required: "1.76.0".to_string(),
            installed: None,
            status: CheckStatus::NotInstalled,
            source: "rust-toolchain.toml".to_string(),
        };
        assert!(r.fix_command().is_some());
    }

    #[test]
    fn compare_versions_works() {
        assert_eq!(compare_versions("1.2.3", "1.2.3"), std::cmp::Ordering::Equal);
        assert_eq!(compare_versions("2.0.0", "1.9.9"), std::cmp::Ordering::Greater);
        assert_eq!(compare_versions("1.0.0", "1.0.1"), std::cmp::Ordering::Less);
    }

    #[test]
    fn compare_versions_different_lengths() {
        assert_eq!(compare_versions("20", "20.0.0"), std::cmp::Ordering::Equal);
        assert_eq!(compare_versions("20.1", "20.0.0"), std::cmp::Ordering::Greater);
    }

    #[test]
    fn version_matches_compound_range() {
        // ">=18 <22" style from package.json
        assert!(version_matches(">=18 <22", "20.0.0"));
        assert!(version_matches(">=18 <22", "18.0.0"));
        assert!(!version_matches(">=18 <22", "22.0.0"));
        assert!(!version_matches(">=18 <22", "16.0.0"));
    }

    #[test]
    fn version_matches_prerelease_suffix() {
        // Pre-release versions: numeric part should still match
        assert!(version_matches("1.76", "1.76.0-nightly"));
    }

    #[test]
    fn version_matches_star_wildcard() {
        assert!(version_matches("3.*", "3.11.4"));
        assert!(!version_matches("3.*", "4.0.0"));
    }

    #[test]
    fn fix_command_python() {
        let r = CheckResult {
            tool: "python".to_string(),
            required: "3.11".to_string(),
            installed: Some("3.10.0".to_string()),
            status: CheckStatus::Fail,
            source: ".python-version".to_string(),
        };
        assert_eq!(
            r.fix_command(),
            Some("pyenv install 3.11 && pyenv local 3.11".to_string())
        );
    }

    #[test]
    fn fix_command_go() {
        let r = CheckResult {
            tool: "go".to_string(),
            required: "1.22.0".to_string(),
            installed: Some("1.21.0".to_string()),
            status: CheckStatus::Fail,
            source: "go.mod".to_string(),
        };
        assert!(r.fix_command().unwrap().contains("go install"));
    }

    #[test]
    fn fix_command_unknown_tool() {
        let r = CheckResult {
            tool: "ruby".to_string(),
            required: "3.2".to_string(),
            installed: None,
            status: CheckStatus::NotInstalled,
            source: ".ruby-version".to_string(),
        };
        assert!(r.fix_command().is_none());
    }
}
