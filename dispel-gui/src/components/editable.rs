use serde::{Deserialize, Serialize};

use dispel_core::Extractor;

/// Describes a single field of an editable record for GUI rendering.
pub struct FieldDescriptor {
    /// Internal field name (used for matching in get_field/set_field).
    pub name: &'static str,
    /// Human-readable label for the GUI.
    pub label: &'static str,
    /// The kind of field (determines how it's edited).
    pub kind: FieldKind,
}

/// The kind of data a field holds, used to determine the appropriate GUI widget.
pub enum FieldKind {
    /// Free-form text input.
    String,
    /// Multi-line text input (text area).
    TextArea,
    /// Integer input (parsed with `str::parse`).
    Integer,
    /// Boolean input (toggle).
    Boolean,
    /// Dropdown selection with custom parsing.
    Enum { variants: &'static [&'static str] },
    /// Dropdown populated from a lookup map at runtime.
    /// The string is a key into the lookups map passed to the view.
    Lookup(&'static str),
}

/// A record type that can be edited in the GUI through a generic editor.
///
/// This trait bridges dispel-core's binary parsing (`Extractor`) with the GUI's
/// string-based editing model. Each field is read/written as a `String`, with
/// parsing handled by the implementation.
pub trait EditableRecord:
    Clone + Default + Serialize + for<'de> Deserialize<'de> + 'static
{
    /// Descriptors for all editable fields, in display order.
    fn field_descriptors() -> &'static [FieldDescriptor];

    /// Read a field's value as a string.
    fn get_field(&self, field: &str) -> String;

    /// Write a field's value from a string. Returns `true` if the value was valid and applied.
    fn set_field(&mut self, field: &str, value: String) -> bool;

    /// Validate a field value. Returns `Some(error_message)` if invalid, `None` if valid.
    fn validate_field(&self, field: &str, value: &str) -> Option<String> {
        let _ = field;
        let _ = value;
        None
    }

    /// Validate all fields and return a list of (field_name, error_message) pairs.
    fn validate_all(&self) -> Vec<(&'static str, String)> {
        let mut errors = Vec::new();
        for descriptor in Self::field_descriptors() {
            let value = self.get_field(descriptor.name);
            if let Some(error) = self.validate_field(descriptor.name, &value) {
                errors.push((descriptor.name, error));
            }
        }
        errors
    }

    /// Format this record for display in the item list.
    fn list_label(&self) -> String;

    /// Format this record for display, optionally using lookup data to resolve IDs to names.
    fn list_label_with_lookups(
        &self,
        lookups: &std::collections::HashMap<String, Vec<(String, String)>>,
    ) -> String {
        let _ = lookups;
        self.list_label()
    }

    /// Title for the detail panel (e.g. "Weapon Details").
    fn detail_title() -> &'static str;

    /// Text shown when no record is selected (e.g. "No weapon selected").
    fn empty_selection_text() -> &'static str;

    /// Label for the save button (e.g. "Save Weapons").
    fn save_button_label() -> &'static str;

    /// Preferred width of the detail panel in pixels.
    fn detail_width() -> f32 {
        320.0
    }
}

/// Blanket implementation: any type implementing `Extractor` + `EditableRecord`
/// gets file I/O through the trait.
pub trait EditableFileRecord: EditableRecord + Extractor {}
impl<T> EditableFileRecord for T where T: EditableRecord + Extractor {}

// ── set_field helpers ────────────────────────────────────────────────────────

/// Set a `String` field.
#[inline]
pub fn set_str(field: &mut String, value: String) -> bool {
    *field = value;
    true
}

/// Set any field that implements `FromStr`.
#[inline]
pub fn set_int<T: std::str::FromStr>(field: &mut T, value: String) -> bool {
    match value.parse() {
        Ok(v) => {
            *field = v;
            true
        }
        Err(_) => false,
    }
}

/// Set an enum field via a `from_name` function (`&str -> Option<T>`).
#[inline]
pub fn set_enum<T>(field: &mut T, value: String, from_name: impl Fn(&str) -> Option<T>) -> bool {
    match from_name(&value) {
        Some(v) => {
            *field = v;
            true
        }
        None => false,
    }
}

/// Set an enum field by parsing the string as `u8` then calling `from_u8`.
#[inline]
pub fn set_u8_enum<T>(field: &mut T, value: String, from_u8: impl Fn(u8) -> Option<T>) -> bool {
    match value.parse::<u8>().ok().and_then(from_u8) {
        Some(v) => {
            *field = v;
            true
        }
        None => false,
    }
}

/// Set an enum field by parsing the string as `i32` then calling `from_i32`.
///
/// Fallback: if the value is not a valid integer, try matching it against the
/// Debug output of all known variants (e.g. `"Light"` → `MapLighting::Light`).
/// This handles the inspector pick-list which sends display names.
#[inline]
pub fn set_i32_enum<T: std::fmt::Debug>(
    field: &mut T,
    value: String,
    from_i32: impl Fn(i32) -> Option<T>,
) -> bool {
    // Fast path: direct integer parse (handles "0", "1", etc.)
    if let Ok(n) = value.parse::<i32>() {
        if let Some(v) = from_i32(n) {
            *field = v;
            return true;
        }
    }
    // Fallback: match against Debug output of known enum variants
    for i in 0..=255i32 {
        if let Some(v) = from_i32(i) {
            if format!("{:?}", v) == value {
                *field = v;
                return true;
            }
        }
    }
    false
}

/// Set an `Option<String>` field; empty string sets to `None`.
#[inline]
pub fn set_opt_str(field: &mut Option<String>, value: String) -> bool {
    *field = if value.is_empty() { None } else { Some(value) };
    true
}

/// Set an `Option<T>` field; empty string sets to `None`.
#[inline]
pub fn set_opt_int<T: std::str::FromStr>(field: &mut Option<T>, value: String) -> bool {
    if value.is_empty() {
        *field = None;
        true
    } else {
        match value.parse() {
            Ok(v) => {
                *field = Some(v);
                true
            }
            Err(_) => false,
        }
    }
}

/// Set an `Option<T>` enum by parsing as `i32` then calling `from_i32`.
///
/// Fallback: if the value is not a valid integer, try matching it against the
/// Debug output of all known variants (e.g. `"Light"` → `MapLighting::Light`).
/// This handles the inspector pick-list which sends display names.
#[inline]
pub fn set_opt_i32_enum<T: std::fmt::Debug>(
    field: &mut Option<T>,
    value: String,
    from_i32: impl Fn(i32) -> Option<T>,
) -> bool {
    if value.is_empty() {
        *field = None;
        true
    } else {
        // Fast path: direct integer parse
        if let Ok(n) = value.parse::<i32>() {
            if let Some(v) = from_i32(n) {
                *field = Some(v);
                return true;
            }
        }
        // Fallback: match against Debug output of known enum variants
        for i in 0..=255i32 {
            if let Some(v) = from_i32(i) {
                if format!("{:?}", v) == value {
                    *field = Some(v);
                    return true;
                }
            }
        }
        false
    }
}

// ── get_field helpers ────────────────────────────────────────────────────────

/// Format any `Debug`-implementor (enum variant name via `{:?}`).
#[inline]
pub fn fmt_enum<T: std::fmt::Debug>(v: &T) -> String {
    format!("{v:?}")
}

/// Format an `Option<T: ToString>`, returning `""` for `None`.
#[inline]
pub fn get_opt_int<T: ToString>(v: Option<T>) -> String {
    v.map_or_else(String::new, |v| v.to_string())
}

/// Format an `Option<T>` with a custom display closure, returning `""` for `None`.
#[inline]
pub fn get_opt_val<T, F: Fn(T) -> String>(v: Option<T>, f: F) -> String {
    v.map_or_else(String::new, f)
}

/// Format a byte slice as a space-separated hex string.
#[inline]
pub fn hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse a space-separated hex string into `Vec<u8>`.
#[inline]
pub fn parse_hex_string(s: &str) -> Option<Vec<u8>> {
    s.split_whitespace()
        .map(|part| u8::from_str_radix(part, 16).ok())
        .collect()
}

/// Parse a space-separated hex string into a fixed-size array `[u8; N]`.
/// Returns `None` if the number of bytes does not match `N`.
#[inline]
pub fn parse_hex_array<const N: usize>(s: &str) -> Option<[u8; N]> {
    let bytes: Vec<u8> = s
        .split_whitespace()
        .map(|part| u8::from_str_radix(part, 16).ok())
        .collect::<Option<_>>()?;
    if bytes.len() == N {
        let mut arr = [0u8; N];
        arr.copy_from_slice(&bytes);
        Some(arr)
    } else {
        None
    }
}

// ── editable_record_fields! macro ─────────────────────────────────────────────

/// Helper: generate descriptor-kind from kind + args (bracket-delimited).
#[macro_export]
#[doc(hidden)]
macro_rules! __er_kind {
    (String, []) => { $crate::components::editable::FieldKind::String };
    (TextArea, []) => { $crate::components::editable::FieldKind::TextArea };
    (Integer, []) => { $crate::components::editable::FieldKind::Integer };
    (Boolean, []) => { $crate::components::editable::FieldKind::Boolean };
    (OptStr, []) => { $crate::components::editable::FieldKind::String };
    (OptInt, []) => { $crate::components::editable::FieldKind::Integer };
    (HexString, []) => { $crate::components::editable::FieldKind::String };
    (Lookup, [$key:expr]) => { $crate::components::editable::FieldKind::Lookup($key) };
    (Enum, [$ty:ty, [$($v:literal),* $(,)?]]) => {
        $crate::components::editable::FieldKind::Enum { variants: &[$($v),*] }
    };
    (Enum, [$ty:ty, Shared($expr:expr)]) => { $expr };
    (i32Enum, [$ty:ty, [$($v:literal),* $(,)?]]) => {
        $crate::components::editable::FieldKind::Enum { variants: &[$($v),*] }
    };
    (DispEnum, [$ty:ty, [$($v:literal),* $(,)?]]) => {
        $crate::components::editable::FieldKind::Enum { variants: &[$($v),*] }
    };
    (DispEnum, [$ty:ty, Shared($expr:expr)]) => { $expr };
}

/// Helper: generate get-field expression from kind + args (bracket-delimited).
#[macro_export]
#[doc(hidden)]
macro_rules! __er_get {
    (String, [], $this:ident, $field:ident) => {
        $this.$field.clone()
    };
    (TextArea, [], $this:ident, $field:ident) => {
        $this.$field.clone()
    };
    (Integer, [], $this:ident, $field:ident) => {
        $this.$field.to_string()
    };
    (Boolean, [], $this:ident, $field:ident) => {
        if $this.$field != 0 {
            "true".into()
        } else {
            "false".into()
        }
    };
    (OptStr, [], $this:ident, $field:ident) => {
        $this.$field.clone().unwrap_or_default()
    };
    (OptInt, [], $this:ident, $field:ident) => {
        $crate::components::editable::get_opt_int($this.$field)
    };
    (HexString, [], $this:ident, $field:ident) => {
        $crate::components::editable::hex_string(&$this.$field)
    };
    (Lookup, [$key:expr], $this:ident, $field:ident) => {
        $this.$field.to_string()
    };
    (Enum, [$ty:ty, $($rest:tt)*], $this:ident, $field:ident) => {
        $crate::components::editable::fmt_enum(&$this.$field)
    };
    (i32Enum, [$ty:ty, [$($v:literal),* $(,)?]], $this:ident, $field:ident) => {
        $this.$field.to_string()
    };
    (DispEnum, [$ty:ty, $($rest:tt)*], $this:ident, $field:ident) => {
        $this.$field.to_string()
    };
}

/// Helper: generate set-field expression from kind + args (bracket-delimited).
#[macro_export]
#[doc(hidden)]
macro_rules! __er_set {
    (String, [], $this:ident, $field:ident, $value:ident) => {
        $crate::components::editable::set_str(&mut $this.$field, $value)
    };
    (TextArea, [], $this:ident, $field:ident, $value:ident) => {
        $crate::components::editable::set_str(&mut $this.$field, $value)
    };
    (Integer, [], $this:ident, $field:ident, $value:ident) => {
        $crate::components::editable::set_int(&mut $this.$field, $value)
    };
    (Boolean, [], $this:ident, $field:ident, $value:ident) => {
        match $value.as_str() {
            "true" | "1" => {
                $this.$field = 1;
                true
            }
            "false" | "0" => {
                $this.$field = 0;
                true
            }
            _ => false,
        }
    };
    (OptStr, [], $this:ident, $field:ident, $value:ident) => {
        $crate::components::editable::set_opt_str(&mut $this.$field, $value)
    };
    (OptInt, [], $this:ident, $field:ident, $value:ident) => {
        $crate::components::editable::set_opt_int(&mut $this.$field, $value)
    };
    (HexString, [], $this:ident, $field:ident, $value:ident) => {
        $crate::components::editable::parse_hex_string(&$value).is_some_and(|v| {
            $this.$field = v;
            true
        })
    };
    (Lookup, [$key:expr], $this:ident, $field:ident, $value:ident) => {
        $crate::components::editable::set_int(&mut $this.$field, $value)
    };
    (Enum, [$ty:ty, $($rest:tt)*], $this:ident, $field:ident, $value:ident) => {
        $crate::components::editable::set_enum(&mut $this.$field, $value, <$ty>::from_name)
    };
    (i32Enum, [$ty:ty, [$($v:literal),* $(,)?]], $this:ident, $field:ident, $value:ident) => {
        $crate::components::editable::set_i32_enum(&mut $this.$field, $value, <$ty>::from_i32)
    };
    (DispEnum, [$ty:ty, $($rest:tt)*], $this:ident, $field:ident, $value:ident) => {
        $crate::components::editable::set_enum(&mut $this.$field, $value, <$ty>::from_name)
    };
}

/// Helper trait that the `editable_record_fields!` macro implements.
/// Generates the triple of `field_descriptors`/`get_field`/`set_field`.
/// `impl_editable_record!` then implements `EditableRecord` delegating to this.
#[doc(hidden)]
pub trait EditableRecordGenerated {
    fn __editable_fields() -> &'static [FieldDescriptor];
    fn __editable_get(&self, f: &str) -> String;
    fn __editable_set(&mut self, f: &str, v: String) -> bool;
}

/// Implement `EditableRecordGenerated` for `$type` — the "data" part of the
/// trait (field_descriptors, get_field, set_field). Pair with `impl_editable_record!`
/// which generates the full `EditableRecord` impl.
///
/// # Syntax
///
/// ```ignore
/// editable_record_fields!(TypeName, {
///     { field = String / "Label:" },
///     { field = TextArea / "Label:" },
///     { field = Integer / "Label:" },
///     { field = Boolean / "Label:" },
///     { field = OptStr / "Label:" },
///     { field = OptInt / "Label:" },
///     { field = HexString / "Label:" },
///     { field = Lookup("key") / "Label:" },
///     { field = Enum(Type, ["v1", "v2"]) / "Label:" },
///     { field = Enum(Type, Shared(CONST)) / "Label:" },
///     { field = i32Enum(Type, ["v1", "v2"]) / "Label:" },
/// });
/// ```
#[macro_export]
macro_rules! editable_record_fields {
    ($type:ty, {
        $( { $name:ident = $kind:ident $( ($($kind_args:tt)*) )? / $label:expr } ),* $(,)?
    }) => {
        impl $crate::components::editable::EditableRecordGenerated for $type {
            fn __editable_fields() -> &'static [$crate::components::editable::FieldDescriptor] {
                &[$(
                    $crate::components::editable::FieldDescriptor {
                        name: stringify!($name),
                        label: $label,
                        kind: $crate::__er_kind!($kind, [ $($($kind_args)*)? ]),
                    },
                )*]
            }

            fn __editable_get(&self, f: &str) -> String {
                match f {
                    $(
                        stringify!($name) => $crate::__er_get!($kind, [ $($($kind_args)*)? ], self, $name),
                    )*
                    _ => String::new(),
                }
            }

            fn __editable_set(&mut self, f: &str, v: String) -> bool {
                match f {
                    $(
                        stringify!($name) => $crate::__er_set!($kind, [ $($($kind_args)*)? ], self, $name, v),
                    )*
                    _ => false,
                }
            }
        }
    };
}

/// Generate delegation for `field_descriptors()` / `get_field()` / `set_field()`
/// inside a manual `impl EditableRecord` block, delegating to `EditableRecordGenerated`.
///
/// # Usage
///
/// ```ignore
/// editable_record_fields!(WeaponItem, { …fields… });
///
/// impl EditableRecord for WeaponItem {
///     crate::editable_record_delegate!();
///     fn list_label(&self) -> String { format!(…) }
///     fn detail_title() -> &'static str { "Weapon Details" }
///     fn empty_selection_text() -> &'static str { "No weapon selected" }
///     fn save_button_label() -> &'static str { "Save Weapons" }
///     fn detail_width() -> f32 { 280.0 }
/// }
/// ```
#[macro_export]
macro_rules! editable_record_delegate {
    () => {
        fn field_descriptors() -> &'static [$crate::components::editable::FieldDescriptor] {
            <Self as $crate::components::editable::EditableRecordGenerated>::__editable_fields()
        }
        fn get_field(&self, f: &str) -> String {
            <Self as $crate::components::editable::EditableRecordGenerated>::__editable_get(self, f)
        }
        fn set_field(&mut self, f: &str, v: String) -> bool {
            <Self as $crate::components::editable::EditableRecordGenerated>::__editable_set(
                self, f, v,
            )
        }
    };
}
