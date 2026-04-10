# AGENTS.md - OpenCode Instructions for heco

## Project Overview
Rust CLI tool for creating and building HarmonyOS projects using `clap`.

## Structure
```
heco/
├── src/
│   ├── main.rs      # CLI entry, command dispatcher
│   ├── new.rs       # heco new subcommand
│   ├── build.rs     # heco build subcommand
│   ├── run.rs       # heco run subcommand
│   ├── check.rs     # heco check subcommand
│   └── test.rs      # heco test subcommand
```

## Key Commands
- **Build**: `cargo build`
- **Run**: `cargo run -- <command>`

## Subcommands
- `heco new <path> [options]` - Create new HarmonyOS project
- `heco build [options]` - Build project modules
- `heco run [options]` - Run on device
- `heco check [options]` - Lint and type check
- `heco test [options]` - Run tests

## Common Patterns
- All subcommand handlers follow `pub(crate) fn handle_<cmd>(args: <Args>)` pattern
- `#[derive(Parser, Debug)]` + `#[command(name)]` for clap configuration
- Use `Option<String>` for optional parameters
- `--quiet` flag for silent mode (check, run, test)
- Avoid importing the same item twice from same module (causes E0252)

## Dependencies
- `clap` (derive feature) - CLI argument parsing
- `anyhow` - Error handling
- `dialoguer` - Interactive prompts (for heco new)

## Notes
- No README file exists
- HarmonyOS-specific knowledge comes from MCP server in `opencode.json`