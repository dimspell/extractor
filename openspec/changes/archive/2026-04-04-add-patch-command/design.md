## Context

The dispel-extractor CLI currently uses a two-level command hierarchy where the `ref` command has 28+ subcommands, one for each file type (weapons, monsters, maps, etc.). Each subcommand calls a `read_*` function from the `references` module and serializes the result to JSON. All 29 file types already implement the `Extractor` trait with both `read_file()` and `save_file()` methods, but the CLI only exposes the read direction.

The `Command` trait (`execute()`, `name()`, `description()`) and `CommandFactory` pattern provide a clean dispatch mechanism. The CLI is built with clap's derive API.

## Goals / Non-Goals

**Goals:**
- Unified `extract` command with auto-detection replacing 28+ `ref` subcommands
- New `patch` command for round-trip JSON → game file writing
- New `validate` and `list` commands for discoverability and safety
- Backward compatibility: old `ref` commands work with deprecation warnings
- AI-generation friendly: predictable command patterns, structured JSON output
- No changes to existing `references/` parsers or `save_file` implementations

**Non-Goals:**
- No changes to file format parsing logic
- No new file type support — only wiring up existing 29 types
- No schema generation or template commands (deferred to future work)
- No changes to `map`, `sprite`, `sound`, `database`, or `test` commands

## Decisions

### 1. File Type Registry as Static Table

**Decision:** Use a static array of `FileType` structs rather than dynamic registration.

```rust
pub struct FileType {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub extensions: &'static [&'static str],
    pub detect_fn: fn(&Path) -> bool,
    pub extract_fn: fn(&Path) -> Result<serde_json::Value>,
    pub patch_fn: fn(&serde_json::Value, &Path) -> Result<()>,
}
```

**Rationale:** All 29 types are known at compile time. A static table is simpler than a builder pattern, has zero runtime registration overhead, and makes the full type list visible in one file.

**Alternatives considered:**
- Dynamic registration via macro — adds complexity with no benefit for fixed set
- Trait-based dispatch on file types — would require changing all reference modules

### 2. Extract/Patch via serde_json::Value Bridge

**Decision:** The registry's `extract_fn` returns `serde_json::Value` and `patch_fn` accepts `serde_json::Value`, rather than using a generic type parameter.

**Rationale:** Each file type has a different record struct. Using `serde_json::Value` as the bridge avoids needing a common trait with associated types. The concrete `read_file()` and `save_file()` calls happen inside the registry's function pointers, keeping type safety within each module.

**Alternatives considered:**
- Generic `Extractor` trait object — Rust doesn't support trait objects with associated types easily
- Enum of all record types — would require a 29-variant enum and massive match statements

### 3. Auto-Detection Strategy: Extension First, Then Content Sniff

**Decision:** Auto-detection checks file extension first (`.db`, `.ini`, `.ref`, `.dlg`, `.scr`, `.pgp`), then uses a lightweight content sniff for ambiguous cases (e.g., `.scr` files used by both quest and message types).

**Rationale:** Extension-based detection is fast and correct for most cases. Content sniffing handles the few ambiguous cases without requiring users to always specify `--type`.

**Implementation:** Each `FileType` has a `detect_fn` that returns true if the file matches. Detection iterates the registry and returns the first match. Users can override with `--type`.

### 4. JSON Output Format: Records Array with Metadata

**Decision:** Extract output uses a simple array of records, not wrapped in metadata:

```json
[
  { "id": 0, "name": "Short Sword", ... },
  { "id": 1, "name": "Long Sword", ... }
]
```

**Rationale:** The PLAN_CLI_REDESIGN.md proposed a `{ "_meta": ..., "records": [...] }` wrapper, but this adds complexity for minimal benefit. The `--pretty` flag handles readability. Metadata can be added later if needed for validation.

**Alternatives considered:**
- Wrapped format with `_meta` — more verbose, breaks simple jq pipelines
- NDJSON — harder to validate as a whole, less familiar to users

### 5. Patch Backup Strategy

**Decision:** `--in-place` creates a `.bak` backup before writing. `--no-backup` skips it. Default behavior without `--in-place` requires explicit `--output` path.

**Rationale:** Safety first — users should always have a way to recover. The `.bak` extension is conventional and easy to clean up.

### 6. Deprecation Strategy for `ref` Command

**Decision:** Keep the `ref` command fully functional but print a stderr deprecation notice on every invocation:

```
Note: 'ref' command is deprecated. Use 'extract' instead:
  dispel-extractor extract --input file.db --type weapons
```

**Rationale:** Zero breaking changes for existing users/scripts while guiding them to the new command. The old command maps directly to the new extract logic.

### 7. Validation Approach: Type Checking Without Full Schema

**Decision:** The `validate` command deserializes JSON into the target record type and reports serde deserialization errors with field-level context, rather than building full JSON Schema validation.

**Rationale:** Building JSON Schema for all 29 types is significant work. Serde deserialization already provides type checking and field-level errors. We can add schema generation later.

**Alternatives considered:**
- Full JSON Schema generation — significant upfront work, can be deferred
- No validation — unsafe for patch workflow

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| Auto-detection false positives for ambiguous extensions (`.scr`) | Content sniffing + `--type` override + clear error messages listing possible types |
| `serde_json::Value` bridge loses type safety at registry boundary | Type safety is preserved within each reference module's extract/patch functions |
| Large JSON files may have memory issues | Use streaming JSON parser if needed; most game files are < 10MB |
| Backup file creation may fail on read-only filesystems | Clear error message with suggestion to use `--output` instead of `--in-place` |
| Deprecation warnings may break scripts that parse stderr | Warnings go to stderr; stdout remains clean JSON for piping |
