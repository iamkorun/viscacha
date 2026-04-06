<p align="center">
  <h1 align="center">viscacha</h1>
  <p align="center">Check your toolchain versions match what the project expects.</p>
</p>

<p align="center">
  <a href="https://github.com/iamkorun/viscacha/actions/workflows/ci.yml"><img src="https://github.com/iamkorun/viscacha/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://crates.io/crates/viscacha"><img src="https://img.shields.io/crates/v/viscacha.svg" alt="crates.io"></a>
  <a href="https://github.com/iamkorun/viscacha/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/iamkorun/viscacha/stargazers"><img src="https://img.shields.io/github/stars/iamkorun/viscacha?style=social" alt="GitHub Stars"></a>
  <a href="https://buymeacoffee.com/iamkorun"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee"></a>
</p>

---

<!-- TODO: Add demo GIF -->

## The Problem

You clone a repo. You run `npm install`. It fails. Why? The project needs Node 20 but you have Node 18. You check `.nvmrc` — sure enough. You switch. Now `cargo build` fails. The project pins Rust 1.76 in `rust-toolchain.toml` but you're on 1.80. You spend 15 minutes hunting down version files scattered across the project before you can actually start working.

## The Solution

**viscacha** scans your project directory for version constraint files across every major ecosystem, checks what's actually installed on your machine, and prints a clean pass/fail table in one command. No more guessing. No more "works on my machine."

```
$ viscacha

 Tool    Required  Installed  Source               Status
 ─────────────────────────────────────────────────────────
 node    20        20.11.0    .nvmrc               ✓
 python  3.11      3.11.4     .python-version      ✓
 rust    1.76.0    1.80.0     rust-toolchain.toml  ✗
 go      1.22.0    1.22.0     go.mod               ✓

3 passing, 1 failing
```

Mismatch? Ask for fix suggestions:

```
$ viscacha --fix

 Tool  Required  Installed  Source               Status
 ───────────────────────────────────────────────────────
 rust  1.76.0    1.80.0     rust-toolchain.toml  ✗

0 passing, 1 failing

Suggested fixes:
  rust: rustup toolchain install 1.76.0 && rustup override set 1.76.0
```

## Quick Start

```bash
cargo install viscacha
```

Then run it in any project directory:

```bash
viscacha
```

That's it.

## Installation

### From crates.io

```bash
cargo install viscacha
```

### From source

```bash
git clone https://github.com/iamkorun/viscacha.git
cd viscacha
cargo install --path .
```

### Binary releases

Pre-built binaries for Linux, macOS, and Windows are available on the [Releases page](https://github.com/iamkorun/viscacha/releases).

## Usage

### Basic check

Scan the current directory for version constraint files and check installed versions:

```bash
viscacha
```

### Check a specific directory

```bash
viscacha --dir /path/to/project
```

### Show fix suggestions

When there are mismatches, `--fix` prints the commands you need to run:

```bash
viscacha --fix
```

### Quiet mode (for CI)

Exit code only, no output. Perfect for CI pipelines:

```bash
viscacha --quiet
```

### Multiple ecosystems at once

Drop a `.nvmrc`, `.python-version`, `rust-toolchain.toml`, and `go.mod` in the same project — viscacha checks them all in one pass:

```
$ viscacha

 Tool    Required  Installed  Source               Status
 ─────────────────────────────────────────────────────────
 node    20        20.11.0    .nvmrc               ✓
 python  3.11      3.11.4     .python-version      ✓
 rust    1.76.0    1.76.0     rust-toolchain.toml  ✓
 go      1.22.0    1.22.0     go.mod               ✓
 node    >=18      20.11.0    package.json         ✓
 npm     >=9       10.2.4     package.json         ✓

6 passing
```

## Supported Files

| File | Ecosystem | What it reads |
|------|-----------|---------------|
| `.nvmrc` | Node.js | Node version (e.g. `20`, `v20.11.0`) |
| `.node-version` | Node.js | Node version |
| `.tool-versions` | asdf/mise | `nodejs`, `python`, `rust`, `golang` entries |
| `rust-toolchain.toml` | Rust | `[toolchain] channel` value |
| `rust-toolchain` | Rust | Plain version string |
| `.python-version` | Python | Python version (e.g. `3.11.4`) |
| `go.mod` | Go | `go` directive version |
| `package.json` | Node.js/npm | `engines.node` and `engines.npm` constraints |

### Version constraint syntax

viscacha understands these constraint formats (commonly found in `package.json` engines):

| Syntax | Meaning | Example |
|--------|---------|---------|
| `20` | Major version prefix match | `20` matches `20.11.0` |
| `3.11` | Major.minor prefix match | `3.11` matches `3.11.4` |
| `1.76.0` | Exact match | `1.76.0` matches `1.76.0` |
| `>=18` | Greater than or equal | `>=18` matches `20.0.0` |
| `<20` | Less than | `<20` matches `18.0.0` |
| `~1.2.3` | Patch-level changes | `~1.2.3` matches `1.2.x` |
| `^1.2.3` | Minor+patch changes | `^1.2.3` matches `1.x.x` |
| `14 \|\| 16 \|\| 18` | OR ranges | Matches any listed version |
| `20.x` | Wildcard | `20.x` matches `20.11.0` |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All checks pass (or no version files found) |
| `1` | One or more version mismatches or tools not installed |
| `2` | Bad input — directory does not exist, parse error, etc. |

## Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--dir <PATH>` | `-d` | Directory to scan (defaults to current directory) |
| `--fix` | | Show suggested fix commands for mismatches |
| `--quiet` | `-q` | Exit code only, no output (useful for CI) |
| `--verbose` | `-v` | Show which version files were detected |
| `--help` | `-h` | Show help text |
| `--version` | `-V` | Show version |

`--quiet` and `--verbose` are mutually exclusive — viscacha will tell you instead of silently picking one.

Set the `NO_COLOR` environment variable to disable colored output.

## Limitations

viscacha checks **comparable numeric versions**. It deliberately skips entries that aren't pinned to a specific version, because there's nothing meaningful to compare:

- `.nvmrc` aliases like `lts/iron`, `latest`
- `.tool-versions` placeholders like `system`, `path:/...`, `ref:branch`
- `rust-toolchain.toml` channels like `stable`, `beta`, `nightly`

If your only version file uses one of these, viscacha will report "no version constraint files found" — that's intentional.

## CI Integration

### GitHub Actions

Add viscacha to your CI pipeline to catch toolchain mismatches before builds fail:

```yaml
name: Toolchain Check
on: [push, pull_request]

jobs:
  check-versions:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install viscacha
        run: cargo install viscacha

      - name: Check toolchain versions
        run: viscacha --quiet
```

Or use it as a pre-build sanity check in an existing workflow:

```yaml
- name: Verify toolchain versions
  run: |
    cargo install viscacha
    viscacha || echo "::warning::Toolchain version mismatch detected"
```

## Features

- **Multi-ecosystem** — Node.js, Python, Rust, Go, npm in one command
- **Smart parsing** — understands semver ranges, wildcards, tilde/caret constraints
- **Fix suggestions** — `--fix` tells you exactly what to run
- **CI-friendly** — `--quiet` mode returns exit codes only
- **Zero config** — just run `viscacha` in your project directory
- **Fast** — written in Rust, scans and checks in milliseconds
- **Portable** — single binary, no runtime dependencies

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Write tests for your changes
4. Ensure all tests pass (`cargo test`)
5. Submit a pull request

## License

[MIT](LICENSE)

---

## Star History

<a href="https://star-history.com/#iamkorun/viscacha&Date">
  <img src="https://api.star-history.com/svg?repos=iamkorun/viscacha&type=Date" alt="Star History Chart" width="600">
</a>

---

<p align="center">
  <a href="https://buymeacoffee.com/iamkorun"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me a Coffee" width="200"></a>
</p>
