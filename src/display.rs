use colored::Colorize;

use crate::checker::{CheckResult, CheckStatus};

/// Print a formatted table of check results.
pub fn print_table(results: &[CheckResult], show_fix: bool) {
    if results.is_empty() {
        println!(
            "{}",
            "No version constraint files found in the current directory.".yellow()
        );
        return;
    }

    // Calculate column widths
    let tool_width = results
        .iter()
        .map(|r| r.tool.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let required_width = results
        .iter()
        .map(|r| r.required.len())
        .max()
        .unwrap_or(8)
        .max(8);
    let installed_width = results
        .iter()
        .map(|r| {
            r.installed
                .as_deref()
                .unwrap_or("not installed")
                .len()
        })
        .max()
        .unwrap_or(9)
        .max(9);
    let source_width = results
        .iter()
        .map(|r| r.source.len())
        .max()
        .unwrap_or(6)
        .max(6);

    // Header
    println!(
        " {:<tool_width$}  {:<required_width$}  {:<installed_width$}  {:<source_width$}  {}",
        "Tool", "Required", "Installed", "Source", "Status",
    );
    let total_width = tool_width + required_width + installed_width + source_width + 16;
    println!(" {}", "─".repeat(total_width));

    // Rows
    for result in results {
        let installed_str = result
            .installed
            .as_deref()
            .unwrap_or("not installed");

        let (status_icon, status_color) = match &result.status {
            CheckStatus::Pass => ("✓", "green"),
            CheckStatus::Fail => ("✗", "red"),
            CheckStatus::NotInstalled => ("✗", "red"),
            CheckStatus::ParseError(msg) => {
                eprintln!("  parse error for {}: {}", result.tool, msg);
                ("?", "yellow")
            }
        };

        let line = format!(
            " {:<tool_width$}  {:<required_width$}  {:<installed_width$}  {:<source_width$}  {}",
            result.tool, result.required, installed_str, result.source, status_icon,
        );

        match status_color {
            "green" => println!("{}", line.green()),
            "red" => println!("{}", line.red()),
            "yellow" => println!("{}", line.yellow()),
            _ => println!("{}", line),
        }
    }

    // Fix suggestions
    if show_fix {
        let fixes: Vec<_> = results
            .iter()
            .filter_map(|r| r.fix_command().map(|cmd| (r.tool.clone(), cmd)))
            .collect();

        if !fixes.is_empty() {
            println!();
            println!("{}", "Suggested fixes:".bold());
            for (tool, cmd) in &fixes {
                println!("  {} {}", format!("{}:", tool).bold(), cmd.cyan());
            }
        }
    }
}

/// Determine the process exit code from results.
pub fn exit_code(results: &[CheckResult]) -> i32 {
    let has_error = results
        .iter()
        .any(|r| matches!(r.status, CheckStatus::ParseError(_)));
    if has_error {
        return 2;
    }

    let has_fail = results.iter().any(|r| {
        matches!(
            r.status,
            CheckStatus::Fail | CheckStatus::NotInstalled
        )
    });
    if has_fail {
        return 1;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::{CheckResult, CheckStatus};

    #[test]
    fn exit_code_all_pass() {
        let results = vec![CheckResult {
            tool: "node".into(),
            required: "20".into(),
            installed: Some("20.11.0".into()),
            status: CheckStatus::Pass,
            source: ".nvmrc".into(),
        }];
        assert_eq!(exit_code(&results), 0);
    }

    #[test]
    fn exit_code_has_fail() {
        let results = vec![
            CheckResult {
                tool: "node".into(),
                required: "20".into(),
                installed: Some("20.11.0".into()),
                status: CheckStatus::Pass,
                source: ".nvmrc".into(),
            },
            CheckResult {
                tool: "python".into(),
                required: "3.11".into(),
                installed: Some("3.10.0".into()),
                status: CheckStatus::Fail,
                source: ".python-version".into(),
            },
        ];
        assert_eq!(exit_code(&results), 1);
    }

    #[test]
    fn exit_code_not_installed() {
        let results = vec![CheckResult {
            tool: "go".into(),
            required: "1.22".into(),
            installed: None,
            status: CheckStatus::NotInstalled,
            source: "go.mod".into(),
        }];
        assert_eq!(exit_code(&results), 1);
    }

    #[test]
    fn exit_code_parse_error() {
        let results = vec![CheckResult {
            tool: "node".into(),
            required: "???".into(),
            installed: Some("20.0.0".into()),
            status: CheckStatus::ParseError("bad version".into()),
            source: ".nvmrc".into(),
        }];
        assert_eq!(exit_code(&results), 2);
    }

    #[test]
    fn exit_code_empty_results() {
        assert_eq!(exit_code(&[]), 0);
    }
}
