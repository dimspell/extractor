## Why

The current CLI has 28+ individual `ref` subcommands (one per file type), making it verbose, hard to discover, and difficult for AI tools to use predictably. There is no way to write JSON data back to game files, preventing a round-trip workflow where users can extract, edit, and re-patch game data. This change introduces a unified `extract` and `patch` command pair with auto-detection, plus `validate` and `list` commands for a complete, symmetric CLI experience.

## What Changes

- **New `extract` command** — replaces all `ref` subcommands with a single `extract --input <file> [--type <type>]` command that auto-detects file type and outputs JSON
- **New `patch` command** — writes JSON data back to game files using existing `save_file` implementations, with `--in-place`, `--dry-run`, and backup support
- **New `validate` command** — validates JSON data against known file format constraints before patching
- **New `list` command** — lists all supported file types with descriptions, extensions, and field info (text and JSON output)
- **Deprecated `ref` command** — old `ref` subcommands still work but show migration hints pointing to `extract`
- **BREAKING**: The `ref` command tree is removed in favor of `extract` (with backward-compatible aliases during transition)

## Capabilities

### New Capabilities

- `extract-command`: Unified file extraction with auto-detection, JSON output, and structured metadata
- `patch-command`: Round-trip patching of game files from JSON with backup, dry-run, and in-place modes
- `validate-command`: JSON validation against file format schemas with field-level error reporting
- `list-command`: Discoverable listing of supported file types with machine-readable JSON output
- `file-type-registry`: Central registry mapping file types to detection patterns, extract functions, and patch functions

### Modified Capabilities

<!-- No existing specs to modify -->

## Impact

- **CLI surface**: Replaces `ref` subcommand tree with `extract`, `patch`, `validate`, `list` commands
- **src/main.rs**: New CLI enum structure, removal of `RefCommands` dispatch
- **src/commands/mod.rs**: New modules (`registry`, `unified`, `info`), removal of `ref_command`
- **src/commands/ref_command.rs**: Deleted or repurposed
- **src/references/**: No changes — all parsers and `save_file` implementations remain untouched
- **Backward compatibility**: Old `ref` commands work with deprecation warnings during transition
