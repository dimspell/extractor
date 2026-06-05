// ── Real implementation (feature "lua") ───────────────────────────────────
#[cfg(feature = "lua")]
mod inner {
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    use crate::inspector::{EncodeFn, InspectorEntry};

    const LUA_POISONED: &str = "Lua interpreter lock poisoned";

    fn new_lua(unsafe_mode: bool) -> Result<mlua::Lua, String> {
        let lua = mlua::Lua::new();
        if !unsafe_mode {
            let globals = lua.globals();
            for key in &[
                "os", "io", "loadfile", "dofile", "require", "package", "load",
            ] {
                let _ = globals.set(*key, mlua::Nil);
            }
        }
        Ok(lua)
    }

    struct DecoderDef {
        name: String,
        category: String,
        description: String,
        min_size: usize,
        decode_id: u32,
        encode_id: Option<u32>,
    }

    pub struct LuaScriptEngine {
        lua: Arc<Mutex<mlua::Lua>>,
        decoders: Vec<DecoderDef>,
        next_id: u32,
    }

    impl LuaScriptEngine {
        pub fn new(unsafe_mode: bool) -> Result<Self, String> {
            let lua = new_lua(unsafe_mode)?;
            let lua = Arc::new(Mutex::new(lua));

            let guard = lua.lock().map_err(|_| LUA_POISONED.to_string())?;
            let globals = guard.globals();
            let decoder_table = guard.create_table().map_err(|e| e.to_string())?;
            globals
                .set("__hexedit_decoders", decoder_table)
                .map_err(|e| e.to_string())?;
            drop(guard);

            Ok(Self {
                lua,
                decoders: Vec::new(),
                next_id: 0,
            })
        }

        pub fn load_script<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
            let path = path.as_ref();
            let code = std::fs::read_to_string(path)
                .map_err(|e| format!("cannot read '{}': {e}", path.display()))?;

            let guard = self.lua.lock().map_err(|_| LUA_POISONED.to_string())?;
            let globals = guard.globals();
            let decoder_table: mlua::Table = globals
                .get("__hexedit_decoders")
                .map_err(|e| e.to_string())?;

            let result: mlua::Value = guard
                .load(&code)
                .set_name(path.to_string_lossy().as_ref())
                .eval()
                .map_err(|e| format!("lua error in '{}': {e}", path.display()))?;

            let tbl = match result {
                mlua::Value::Table(t) => t,
                other => {
                    return Err(format!(
                        "script '{}' must return a table, got {:?}",
                        path.display(),
                        other.type_name()
                    ));
                }
            };

            let name: String = tbl
                .get("name")
                .map_err(|_| format!("'{}': decoder missing 'name'", path.display()))?;
            let min_size: usize = tbl
                .get("min_size")
                .map_err(|_| format!("'{}': decoder missing 'min_size'", path.display()))?;
            let category: String = tbl.get("category").unwrap_or("Custom".to_string());
            let description: String = tbl.get("description").unwrap_or_default();

            let decode_fn: mlua::Function = tbl.get("decode").map_err(|_| {
                format!(
                    "'{}': decoder '{name}' missing 'decode' function",
                    path.display()
                )
            })?;
            let encode_fn: Option<mlua::Function> = tbl.get("encode").ok();

            let decode_id = self.next_id;
            self.next_id += 1;
            decoder_table
                .set(decode_id, decode_fn)
                .map_err(|e| format!("failed to store decode function for '{name}': {e}"))?;

            let encode_id = encode_fn.and_then(|f| {
                let id = self.next_id;
                self.next_id += 1;
                decoder_table.set(id, f).ok()?;
                Some(id)
            });

            self.decoders.push(DecoderDef {
                name,
                category,
                description,
                min_size,
                decode_id,
                encode_id,
            });

            Ok(())
        }

        pub fn entries(&self) -> Vec<InspectorEntry> {
            let lua = self.lua.clone();
            self.decoders
                .iter()
                .map(|def| {
                    let lua_decode = lua.clone();
                    let lua_encode = lua.clone();
                    let decode_id = def.decode_id;
                    InspectorEntry {
                        name: def.name.clone(),
                        min_size: def.min_size,
                        decode: Box::new(move |bytes: &[u8]| {
                            let guard = match lua_decode.lock() {
                                Ok(g) => g,
                                Err(_) => return "—".to_string(),
                            };
                            let globals = guard.globals();
                            let table: mlua::Table = match globals.get("__hexedit_decoders") {
                                Ok(t) => t,
                                Err(_) => return "—".to_string(),
                            };
                            let func: mlua::Function = match table.get(decode_id) {
                                Ok(f) => f,
                                Err(_) => return "—".to_string(),
                            };
                            // NOTE: must call `mlua::String::wrap()` to pass bytes as a
                            // *Lua string* — passing `Vec<u8>` directly creates a Lua
                            // *table* of numbers (generic `IntoLua for Vec<T>`) which
                            // breaks `bytes:byte(i)` calls in Lua decoder scripts.
                            match func.call::<mlua::String>(mlua::String::wrap(bytes)) {
                                Ok(s) => s.to_string_lossy().to_string(),
                                Err(e) => format!("— ({e})"),
                            }
                        }),
                        encode: def.encode_id.map(|encode_id| -> EncodeFn {
                            Box::new(move |s: &str| {
                                let guard =
                                    lua_encode.lock().map_err(|_| LUA_POISONED.to_string())?;
                                let globals = guard.globals();
                                let table: mlua::Table = globals
                                    .get("__hexedit_decoders")
                                    .map_err(|e| e.to_string())?;
                                let func: mlua::Function =
                                    table.get(encode_id).map_err(|e| e.to_string())?;
                                func.call::<mlua::String>(s)
                                    .map(|r| r.as_bytes().to_vec())
                                    .map_err(|e| e.to_string())
                            })
                        }),
                        category: def.category.clone(),
                        description: def.description.clone(),
                    }
                })
                .collect()
        }
    }
}

// ── Stub (feature "lua" not enabled) ──────────────────────────────────────
#[cfg(not(feature = "lua"))]
mod inner {
    use crate::inspector::InspectorEntry;
    use std::path::Path;

    pub struct LuaScriptEngine;

    impl LuaScriptEngine {
        pub fn new(_unsafe_mode: bool) -> Result<Self, String> {
            Ok(Self)
        }

        pub fn load_script<P: AsRef<Path>>(&mut self, _path: P) -> Result<(), String> {
            Ok(())
        }

        pub fn entries(&self) -> Vec<InspectorEntry> {
            Vec::new()
        }
    }
}

pub use inner::LuaScriptEngine;

impl Default for LuaScriptEngine {
    fn default() -> Self {
        Self::new(false).unwrap_or_else(|_| panic!("LuaScriptEngine::default() failed"))
    }
}
