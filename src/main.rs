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
#[command(name = "viscacha", version, about)]
struct Cli {
    /// Directory to scan (defaults to current directory)
    #[arg(short, long)]
    dir: Option<PathBuf>,

    /// Show suggested fix commands for mismatches
    #[arg(long)]
    fix: bool,

    /// Quiet mode: exit code only, no output (useful for CI)
    #[arg(short, long)]
    quiet: bool,
}

fn main() {
    let cli = Cli::parse();

    let dir = cli.dir.unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });

    let files = detector::detect_version_files(&dir);

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
