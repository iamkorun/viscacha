use std::path::PathBuf;
use std::process;

use clap::Parser;

use viscacha::checker;
use viscacha::detector;
use viscacha::display;
use viscacha::parser;

/// viscacha — toolchain version checker
///
/// Reads version constraint files in the current directory and checks
/// what's installed vs what's required, printing a clean pass/fail table.
#[derive(Parser, Debug)]
#[command(
    name = "viscacha",
    version,
    about = "Check your toolchain versions match what the project expects",
    long_about = "viscacha scans your project directory for version constraint files \
                  (.nvmrc, .tool-versions, rust-toolchain.toml, .python-version, go.mod, \
                  package.json) and checks what's actually installed on your machine.",
    after_long_help = "EXAMPLES:
    viscacha                      Check the current directory
    viscacha --dir ../api         Check a specific directory
    viscacha --fix                Print suggested fix commands for mismatches
    viscacha --quiet              Exit code only — useful in CI pipelines
    viscacha --verbose            Show which version files were detected

EXIT CODES:
    0   All checks pass (or no version files found)
    1   One or more version mismatches or tools not installed
    2   Bad input — directory does not exist, parse error, etc.

NO_COLOR:
    Set the NO_COLOR environment variable to disable colored output."
)]
struct Cli {
    /// Directory to scan (defaults to current directory)
    #[arg(short, long, value_name = "PATH")]
    dir: Option<PathBuf>,

    /// Show suggested fix commands for mismatches
    #[arg(long)]
    fix: bool,

    /// Quiet mode — no output, exit code only (useful for CI)
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,

    /// Show which version files were detected
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    let dir = cli.dir.unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });

    // Validate the target before scanning so users get a clear error
    // instead of a silent "no files found" message.
    if !dir.exists() {
        eprintln!("error: {} does not exist", dir.display());
        process::exit(2);
    }
    if !dir.is_dir() {
        eprintln!("error: {} is not a directory", dir.display());
        process::exit(2);
    }

    let files = detector::detect_version_files(&dir);

    if cli.verbose {
        use colored::Colorize;
        eprintln!("{} scanning {}", "verbose:".dimmed(), dir.display());
        if files.is_empty() {
            eprintln!("{} no version files found", "verbose:".dimmed());
        } else {
            for f in &files {
                eprintln!("{} found {}", "verbose:".dimmed(), f.display());
            }
        }
    }

    let requirements: Vec<_> = files
        .iter()
        .flat_map(|f| parser::parse_version_file(f))
        .collect();

    let results = checker::check_all(&requirements);

    if !cli.quiet {
        display::print_table(&results, cli.fix);
    }

    let code = display::exit_code(&results);
    process::exit(code);
}
