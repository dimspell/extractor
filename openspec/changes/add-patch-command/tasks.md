## 1. File Type Registry

- [x] 1.1 Create `src/commands/registry.rs` with `FileType` struct definition
- [x] 1.2 Implement detection functions for all 29 file types (extension + content sniffing)
- [x] 1.3 Implement extract bridge functions wrapping each `Extractor::read_file()` → `serde_json::Value`
- [x] 1.4 Implement patch bridge functions wrapping `serde_json::Value` → `Extractor::save_file()`
- [x] 1.5 Build static `FILE_TYPES` registry array with all 29 entries
- [x] 1.6 Implement `detect(path)` function that iterates registry and returns matching type
- [x] 1.7 Implement `get_by_key(key)` function for type override lookup
- [x] 1.8 Add registry module to `src/commands/mod.rs`

## 2. Extract Command

- [x] 2.1 Create `src/commands/unified.rs` with `ExtractCommand` struct and clap args
- [x] 2.2 Implement extract execution: resolve type, call extract bridge, serialize to JSON
- [x] 2.3 Implement `--output` flag for file output vs stdout
- [x] 2.4 Implement `--pretty` flag for indented JSON
- [x] 2.5 Implement `--type` flag for type override
- [x] 2.6 Implement error handling: file not found, unknown type, parse failure
- [x] 2.7 Add extract command module to `src/commands/mod.rs`

## 3. Patch Command

- [x] 3.1 Add `PatchCommand` struct to `src/commands/unified.rs` with clap args
- [x] 3.2 Implement patch execution: read JSON, resolve type, call patch bridge
- [x] 3.3 Implement `--output` flag for alternate output path
- [x] 3.4 Implement `--dry-run` flag: validate without writing
- [x] 3.5 Implement `--in-place` flag with `.bak` backup creation
- [x] 3.6 Implement `--no-backup` flag to skip backup
- [x] 3.7 Implement error handling: JSON parse, validation, permissions, backup failure
- [x] 3.8 Add patch command module to `src/commands/mod.rs`

## 4. Validate Command

- [x] 4.1 Create `src/commands/info.rs` with `ValidateCommand` struct and clap args
- [x] 4.2 Implement validate execution: read JSON, deserialize into target record type
- [x] 4.3 Implement `--verbose` flag for detailed field-level error output
- [x] 4.4 Implement structured JSON error output for validation failures
- [x] 4.5 Add validate command module to `src/commands/mod.rs`

## 5. List Command

- [x] 5.1 Add `ListCommand` struct to `src/commands/info.rs` with clap args
- [x] 5.2 Implement default text table output showing all file types
- [x] 5.3 Implement `--format json` flag for machine-readable output
- [x] 5.4 Implement `--filter` flag for case-insensitive type filtering
- [x] 5.5 Add list command module to `src/commands/mod.rs`

## 6. Refactor main.rs

- [x] 6.1 Add new `Extract`, `Patch`, `Validate`, `List` variants to `Commands` enum
- [x] 6.2 Add `ExtractArgs`, `PatchArgs`, `ValidateArgs`, `ListArgs` clap structs
- [x] 6.3 Add dispatch arms in `main()` for new commands
- [x] 6.4 Add `CommandFactory` methods for new commands
- [x] 6.5 Add deprecation warning to `ref` command execution (stderr message)
- [x] 6.6 Update `ref` command help text to point to `extract`

## 7. Cleanup

- [x] 7.1 Remove `RefCommands` enum and `RefArgs` from `ref_command.rs`
- [x] 7.2 Remove `create_ref_command` from `CommandFactory` or repurpose
- [x] 7.3 Update `src/commands/mod.rs` to export new modules, remove old ref exports
- [x] 7.4 Run `cargo build` and fix any compilation errors
- [x] 7.5 Run `cargo test` and fix any test failures

## 8. Verification

- [x] 8.1 Test round-trip: extract a .db file, patch it back, verify byte-identical
- [x] 8.2 Test round-trip: extract a .ini file, patch it back, verify byte-identical
- [x] 8.3 Test round-trip: extract a .ref file, patch it back, verify byte-identical
- [x] 8.4 Test `--dry-run` with valid and invalid JSON
- [x] 8.5 Test `--in-place` creates .bak backup correctly
- [x] 8.6 Test `validate` with valid and invalid JSON for multiple types
- [x] 8.7 Test `list` command text and JSON output
- [x] 8.8 Test `list --filter` with matching and non-matching patterns
- [x] 8.9 Test deprecation warning on `ref` command
- [x] 8.10 Test auto-detection for all file extensions
