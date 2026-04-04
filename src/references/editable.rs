use serde::{Deserialize, Serialize};

use super::extractor::Extractor;

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
    /// Integer input (parsed with `str::parse`).
    Integer,
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
    /// Default implementation iterates through field_descriptors.
    fn validate_all(&self) -> Vec<(&'static str, String)> {
        let mut errors = Vec::new();
        let descriptors = Self::field_descriptors();
        for descriptor in descriptors {
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
    /// Default implementation delegates to `list_label()`.
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
