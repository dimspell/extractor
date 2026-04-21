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
#[inline]
pub fn set_i32_enum<T>(field: &mut T, value: String, from_i32: impl Fn(i32) -> Option<T>) -> bool {
    match value.parse::<i32>().ok().and_then(from_i32) {
        Some(v) => {
            *field = v;
            true
        }
        None => false,
    }
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
#[inline]
pub fn set_opt_i32_enum<T>(
    field: &mut Option<T>,
    value: String,
    from_i32: impl Fn(i32) -> Option<T>,
) -> bool {
    if value.is_empty() {
        *field = None;
        true
    } else {
        match value.parse::<i32>().ok().and_then(from_i32) {
            Some(v) => {
                *field = Some(v);
                true
            }
            None => false,
        }
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
