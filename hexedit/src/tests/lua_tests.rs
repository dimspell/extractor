use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ui::coloring::ColorScheme;
use crate::LuaScriptEngine;

/// Write a Lua script to the temp dir and return its path.
/// Uses a global counter to keep paths unique across tests.
static SCRIPT_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Helper: create default pane grid + focus for test constructors.
fn default_test_panes() -> (iced::widget::pane_grid::State<crate::domain::panel::HexPanel>, iced::widget::pane_grid::Pane) {
    let panes = crate::domain::panel::default_pane_grid();
    let focus = *panes.iter().next().map(|(id, _)| id).unwrap();
    (panes, focus)
}

fn write_script(dir: &str, name: &str, code: &str) -> PathBuf {
    let counter = SCRIPT_COUNTER.fetch_add(1, Ordering::SeqCst);
    let d = std::env::temp_dir()
        .join("hexedit_lua_test")
        .join(dir)
        .join(counter.to_string());
    std::fs::create_dir_all(&d).expect("create temp dir for lua script");
    let path = d.join(name);
    std::fs::write(&path, code).expect("write lua test script");
    path
}

// ── Lifecycle ─────────────────────────────────────────────────────────

#[test]
fn test_lua_engine_create() {
    let engine = LuaScriptEngine::new(false);
    assert!(engine.is_ok(), "engine should create in safe mode");
    let engine = LuaScriptEngine::new(true);
    assert!(engine.is_ok(), "engine should create in unsafe mode");
    let engine = LuaScriptEngine::default();
    let entries = engine.entries();
    assert!(entries.is_empty(), "default engine has no entries");
}

/// Shared Lua script for basic decode testing.
const BASIC_SCRIPT: &str = r#"
return {
    name = "test_decoder",
    min_size = 2,
    category = "Test",
    description = "A test decoder",
    decode = function(bytes)
        return string.format("%02X %02X", bytes:byte(1), bytes:byte(2))
    end,
    encode = function(s)
        return s
    end,
}
"#;

#[test]
fn test_lua_engine_load_valid_script() {
    let path = write_script("load_valid", "test.lua", BASIC_SCRIPT);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    assert_eq!(entries.len(), 1, "should have 1 entry");
    assert_eq!(entries[0].name, "test_decoder");
    assert_eq!(entries[0].min_size, 2);
    assert_eq!(entries[0].category, "Test");
    assert_eq!(entries[0].description, "A test decoder");
}

#[test]
fn test_lua_engine_decode_basic() {
    let path = write_script("decode_basic", "test.lua", BASIC_SCRIPT);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    let result = (entries[0].decode)(&[0xDE, 0xAD]);
    assert_eq!(result, "DE AD", "decode should format two hex bytes");
}

#[test]
fn test_lua_engine_decode_null_bytes() {
    let path = write_script("decode_null", "test.lua", BASIC_SCRIPT);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    let result = (entries[0].decode)(&[0x00, 0x00]);
    assert_eq!(result, "00 00", "decode should handle null bytes");
}

#[test]
fn test_lua_engine_decode_multi_byte() {
    let path = write_script("decode_multi", "test.lua", BASIC_SCRIPT);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    let result = (entries[0].decode)(&[0x01, 0x02, 0x03, 0x04]);
    // Script with min_size=2 only formats first 2 bytes
    assert_eq!(result, "01 02", "decode should handle 4 bytes, but only formats first 2");
}

#[test]
fn test_lua_engine_encode_returns_bytes() {
    let path = write_script("encode_bytes", "test.lua", r#"
return {
    name = "echo",
    min_size = 1,
    decode = function(bytes)
        return string.format("%02X", bytes:byte(1))
    end,
    encode = function(s)
        return s
    end,
}
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    let encode = entries[0].encode.as_ref().expect("should have encode");
    let result = encode("hello").expect("encode should succeed");
    assert_eq!(result, b"hello", "encode should echo input string");
}

#[test]
fn test_lua_engine_encode_hex_string() {
    let path = write_script("encode_hex", "test.lua", r#"
return {
    name = "hex_encode",
    min_size = 1,
    decode = function(bytes)
        return string.format("%02X", bytes:byte(1))
    end,
    encode = function(s)
        -- Convert hex string like "FF" back to bytes
        return (s:gsub("(%x%x)", function(h)
            return string.char(tonumber(h, 16))
        end))
    end,
}
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    let encode = entries[0].encode.as_ref().expect("should have encode");
    let result = encode("DEAD").expect("encode should succeed");
    assert_eq!(result, &[0xDE, 0xAD], "encode should convert hex to bytes");
}

#[test]
fn test_lua_engine_encode_byte_calculation() {
    let path = write_script("encode_calc", "test.lua", r#"
return {
    name = "twos_complement",
    min_size = 1,
    decode = function(bytes)
        local b = bytes:byte(1)
        if b > 127 then
            return tostring(b - 256)
        else
            return tostring(b)
        end
    end,
    encode = function(s)
        local n = tonumber(s)
        if n < 0 then n = n + 256 end
        return string.char(n)
    end,
}
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();

    // Decode 0x80 → should be -128
    assert_eq!((entries[0].decode)(&[0x80]), "-128");
    // Decode 0x7F → should be 127
    assert_eq!((entries[0].decode)(&[0x7F]), "127");

    // Encode "-1" → should write 0xFF
    let result = entries[0]
        .encode
        .as_ref()
        .unwrap()("-1")
        .expect("encode should succeed");
    assert_eq!(result, &[0xFF]);
}

// ── Multiple scripts ──────────────────────────────────────────────────

#[test]
fn test_lua_engine_multiple_scripts() {
    let p1 = write_script("multi", "d1.lua", r#"
return { name = "d1", min_size = 1, decode = function(b) return "first" end }
"#);
    let p2 = write_script("multi", "d2.lua", r#"
return { name = "d2", min_size = 2, decode = function(b) return "second" end }
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&p1).unwrap();
    engine.load_script(&p2).unwrap();
    let entries = engine.entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name, "d1");
    assert_eq!(entries[1].name, "d2");
    assert_eq!((entries[0].decode)(&[0x00]), "first");
    assert_eq!((entries[1].decode)(&[0x00, 0x00]), "second");
}

#[test]
fn test_lua_engine_entries_independent_from_engine() {
    let path = write_script("independent", "test.lua", r#"
return { name = "indep", min_size = 1, decode = function(b) return "ok" end }
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    drop(engine); // Drop the engine — entries should still work
    assert_eq!((entries[0].decode)(&[0x00]), "ok");
}

// ── Error handling ────────────────────────────────────────────────────

#[test]
fn test_lua_engine_missing_name_returns_error() {
    let path = write_script("missing_name", "test.lua", r#"
return { min_size = 1, decode = function(b) return "x" end }
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    let err = engine.load_script(&path).unwrap_err();
    assert!(err.contains("name"), "error should mention 'name': {err}");
}

#[test]
fn test_lua_engine_missing_min_size_returns_error() {
    let path = write_script("missing_min_size", "test.lua", r#"
return { name = "x", decode = function(b) return "x" end }
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    let err = engine.load_script(&path).unwrap_err();
    assert!(err.contains("min_size"), "error should mention 'min_size': {err}");
}

#[test]
fn test_lua_engine_missing_decode_returns_error() {
    let path = write_script("missing_decode", "test.lua", r#"
return { name = "x", min_size = 1 }
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    let err = engine.load_script(&path).unwrap_err();
    assert!(err.contains("decode"), "error should mention 'decode': {err}");
}

#[test]
fn test_lua_engine_not_a_table_returns_error() {
    let path = write_script("not_table", "test.lua", r#"return 42"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    let err = engine.load_script(&path).unwrap_err();
    assert!(err.contains("table"), "error should mention 'table': {err}");
}

#[test]
fn test_lua_engine_invalid_syntax_returns_error() {
    let path = write_script("bad_syntax", "test.lua", "this is not valid lua");
    let mut engine = LuaScriptEngine::new(false).unwrap();
    let err = engine.load_script(&path).unwrap_err();
    // Should mention a Lua syntax error
    assert!(
        err.contains("syntax") || err.contains("error"),
        "error should describe syntax problem: {err}"
    );
}

#[test]
fn test_lua_engine_nonexistent_file_returns_error() {
    let mut engine = LuaScriptEngine::new(false).unwrap();
    let err = engine
        .load_script("/tmp/nonexistent_script_12345.lua")
        .unwrap_err();
    assert!(err.contains("cannot read"), "error should mention file read: {err}");
}

#[test]
fn test_lua_engine_decode_runtime_error_graceful() {
    let path = write_script("decode_error", "test.lua", r#"
return {
    name = "faulty",
    min_size = 1,
    decode = function(bytes)
        error("something went wrong in lua")
    end,
}
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    let result = (entries[0].decode)(&[0x00]);
    assert!(
        result.starts_with("—"),
        "decode error should produce '—', got: {result:?}"
    );
    assert!(
        result.contains("something went wrong"),
        "error detail should be included: {result}"
    );
}

#[test]
fn test_lua_engine_encode_runtime_error_graceful() {
    let path = write_script("encode_error", "test.lua", r#"
return {
    name = "faulty_encode",
    min_size = 1,
    decode = function(b) return "x" end,
    encode = function(s)
        error("encode failure")
    end,
}
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    let encode = entries[0].encode.as_ref().unwrap();
    let result = encode("test");
    assert!(result.is_err(), "encode should return Err on lua error");
    assert!(
        result.unwrap_err().contains("encode failure"),
        "error detail should be preserved"
    );
}

#[test]
fn test_lua_engine_decode_returns_non_string() {
    let path = write_script("decode_non_string", "test.lua", r#"
return {
    name = "returns_number",
    min_size = 1,
    decode = function(bytes)
        return 42
    end,
}
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    let result = (entries[0].decode)(&[0x00]);
    // mlua's FromLua for String coerces numbers — 42 becomes "42".
    assert_eq!(result, "42", "mlua coerces number to string");
}

#[test]
fn test_lua_engine_encode_returns_non_string() {
    let path = write_script("encode_non_string", "test.lua", r#"
return {
    name = "encode_number",
    min_size = 1,
    decode = function(b) return "x" end,
    encode = function(s)
        return 42  -- return number, not string
    end,
}
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    let result = entries[0].encode.as_ref().unwrap()("test").unwrap();
    // mlua coerces number 42 → string "42" → bytes b"42"
    assert_eq!(result, b"42", "mlua coerces number to string");
}

// ── Unsafe/safe mode ──────────────────────────────────────────────────

#[test]
fn test_lua_engine_unsafe_mode_blocks_os() {
    let path = write_script("unsafe_os", "test.lua", r#"
return {
    name = "os_user",
    min_size = 1,
    decode = function(bytes)
        return tostring(os.clock())
    end,
}
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    let result = (entries[0].decode)(&[0x00]);
    assert!(
        result.starts_with("—"),
        "accessing nilled 'os' should fail: {result:?}"
    );
}

#[test]
fn test_lua_engine_unsafe_mode_allows_os() {
    let path = write_script("unsafe_allowed", "test.lua", r#"
return {
    name = "os_user",
    min_size = 1,
    decode = function(bytes)
        if os and os.clock then
            return "unsafe"
        else
            return "safe"
        end
    end,
}
"#);
    let mut engine = LuaScriptEngine::new(true).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    let result = (entries[0].decode)(&[0x00]);
    assert_eq!(result, "unsafe", "unsafe mode should allow os.clock");
}

// ── Script with custom category and description ───────────────────────

#[test]
fn test_lua_engine_default_category_and_description() {
    let path = write_script("default_cat", "test.lua", r#"
return { name = "minimal", min_size = 1, decode = function(b) return "ok" end }
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    assert_eq!(entries[0].category, "Custom", "default category");
    assert_eq!(entries[0].description, "", "default description");
}

// ── Load scripts from directory (HexEditorState::load_lua_scripts) ────

#[test]
fn test_load_lua_scripts_from_dir() {
    let counter = SCRIPT_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir()
        .join("hexedit_lua_test")
        .join("load_dir")
        .join(counter.to_string());
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::write(dir.join("a.lua"), r#"
return { name = "from_dir_a", min_size = 1, decode = function(b) return "A" end }
"#).unwrap();
    std::fs::write(dir.join("b.lua"), r#"
return { name = "from_dir_b", min_size = 1, decode = function(b) return "B" end }
"#).unwrap();
    // Non-.lua file should be ignored
    std::fs::write(dir.join("notes.txt"), "not a script").unwrap();

    let (panes, pane_focus) = default_test_panes();
    let mut state = crate::state::HexEditorState {
        path: std::path::PathBuf::from("test.bin"),
        name: "test.bin".to_string(),
        panes,
        pane_focus,
        provider: crate::provider::BufferProvider::from_bytes(vec![0x00]),
        bytes_per_row: 16,
        selection: crate::selection::Selection::single(0),
        edit_mode: None,
        inspector_edit: None,
        vanilla: None,
        vanilla_diff: std::collections::BTreeSet::new(),
        patterns: Vec::new(),
        pattern_by_addr: std::collections::BTreeMap::new(),
        show_pattern_list: false,
        next_pattern_id: 0,
        groups: Vec::new(),
        next_group_id: 0,
        collapsed_groups: std::collections::BTreeSet::new(),
        context_menu_addr: None,
        goto: None,
        search: crate::search::SearchState::new(),
        show_decimal: false,
        status_msg: String::new(),
        error: None,
        cache: gui_widgets::components::paragraph_cache::ParagraphCache::default(),
        lua_engine: LuaScriptEngine::new(false).unwrap(),
        export_config: None,
        repeat_pattern: None,
        row_annotations: std::collections::BTreeMap::new(),
        active_patterns: std::collections::BTreeSet::new(),
        renaming_group: None,
        renaming_group_draft: String::new(),
        color_scheme: ColorScheme::Monochrome,
        dim_nulls: false,
        settings_open: false,
    };
    let errors = state.load_lua_scripts(&dir);
    assert!(errors.is_empty(), "should load without errors: {errors:?}");
    let entries = state.lua_engine.entries();
    assert_eq!(entries.len(), 2, "should load 2 scripts");
    assert_eq!(entries[0].name, "from_dir_a");
    assert_eq!(entries[1].name, "from_dir_b");
}

#[test]
fn test_load_lua_scripts_nonexistent_dir_returns_no_errors() {
    // Current behavior: non-existent dir returns empty errors (treated as
    // "no scripts to load", not an error condition).
    let engine = LuaScriptEngine::new(false).unwrap();
    let (panes, pane_focus) = default_test_panes();
    let mut state = crate::state::HexEditorState {
        path: std::path::PathBuf::from("test.bin"),
        name: "test.bin".to_string(),
        panes,
        pane_focus,
        provider: crate::provider::BufferProvider::from_bytes(vec![0x00]),
        bytes_per_row: 16,
        selection: crate::selection::Selection::single(0),
        edit_mode: None,
        inspector_edit: None,
        vanilla: None,
        vanilla_diff: std::collections::BTreeSet::new(),
        patterns: Vec::new(),
        pattern_by_addr: std::collections::BTreeMap::new(),
        show_pattern_list: false,
        next_pattern_id: 0,
        groups: Vec::new(),
        next_group_id: 0,
        collapsed_groups: std::collections::BTreeSet::new(),
        context_menu_addr: None,
        goto: None,
        search: crate::search::SearchState::new(),
        show_decimal: false,
        status_msg: String::new(),
        error: None,
        cache: gui_widgets::components::paragraph_cache::ParagraphCache::default(),
        lua_engine: engine,
        export_config: None,
        repeat_pattern: None,
        row_annotations: std::collections::BTreeMap::new(),
        active_patterns: std::collections::BTreeSet::new(),
        renaming_group: None,
        renaming_group_draft: String::new(),
        color_scheme: ColorScheme::Monochrome,
        dim_nulls: false,
        settings_open: false,
    };
    let errors = state.load_lua_scripts(&std::path::PathBuf::from("/nonexistent/lua/dir"));
    assert!(errors.is_empty(), "non-existent dir should return 0 errors");
}

// ── Lua + iced_test integration: verify decoders appear in inspector ──

#[test]
fn test_lua_decoder_appears_in_inspector_view() {
    use crate::view::view;
    use iced_test::simulator;

    let path = write_script("inspector_view", "test.lua", r#"
return {
    name = "lua_decoder",
    min_size = 1,
    category = "LuaScripts",
    description = "A Lua decoder",
    decode = function(bytes)
        return string.format("LUA:0x%02X", bytes:byte(1))
    end,
}
"#);
    let mut engine = LuaScriptEngine::new(false).unwrap();
    engine.load_script(&path).unwrap();
    let entries = engine.entries();
    assert_eq!(entries.len(), 1, "should have Lua decoder");

    // Build a minimal state that includes the Lua entries.
    // We need to trick the inspector view into showing Lua entries.
    // The inspector reads from the HexEditorState via the entries() method.
    // But entries() returns only the ENGINE's entries. The inspector ALSO
    // renders the built-in ENTRIES (from inspector.rs). Lua entries are
    // NOT automatically rendered in the inspector view — the view only
    // shows built-in ENTRIES and config.extra_entries.
    //
    // So to see Lua entries in the view, they must be in config.extra_entries.
    // This won't be visible directly from the LuaScriptEngine alone.
    //
    // Instead, let's verify that Lua entries decode correctly and that
    // a custom InspectorEntry built from Lua can be used in the view.
    let (panes, pane_focus) = default_test_panes();
    let state = crate::state::HexEditorState {
        path: std::path::PathBuf::from("test.bin"),
        name: "test.bin".to_string(),
        panes,
        pane_focus,
        provider: crate::provider::BufferProvider::from_bytes(vec![0xAB]),
        bytes_per_row: 16,
        selection: crate::selection::Selection::single(0),
        edit_mode: None,
        inspector_edit: None,
        vanilla: None,
        vanilla_diff: std::collections::BTreeSet::new(),
        patterns: Vec::new(),
        pattern_by_addr: std::collections::BTreeMap::new(),
        show_pattern_list: false,
        next_pattern_id: 0,
        groups: Vec::new(),
        next_group_id: 0,
        collapsed_groups: std::collections::BTreeSet::new(),
        context_menu_addr: None,
        goto: None,
        search: crate::search::SearchState::new(),
        show_decimal: false,
        status_msg: String::new(),
        error: None,
        cache: gui_widgets::components::paragraph_cache::ParagraphCache::default(),
        lua_engine: engine,
        export_config: None,
        repeat_pattern: None,
        row_annotations: std::collections::BTreeMap::new(),
        active_patterns: std::collections::BTreeSet::new(),
        renaming_group: None,
        renaming_group_draft: String::new(),
        color_scheme: ColorScheme::Monochrome,
        dim_nulls: false,
        settings_open: false,
    };
    // Verify the decode works
    assert_eq!((entries[0].decode)(&[0xAB]), "LUA:0xAB");
    // Verify it can be passed as extra_entries in config
    let config = crate::config::HexEditorConfig {
        extra_entries: entries,
        ..Default::default()
    };
    let mut ui = simulator(view(&state, &config));
    ui.find("lua_decoder").expect("Lua decoder name should appear in inspector");
    ui.find("LUA:0xAB").expect("Lua decoder value should appear in inspector");
}
