use std::collections::{HashMap, HashSet};

/// Owned labels and id mapping for a lookup-backed select box.
#[derive(Debug, Clone)]
pub struct LookupSelect {
    /// Labels in lookup order, with an optional fallback for a missing id.
    pub options: Vec<String>,
    /// Label to render as the current selection.
    pub selected: Option<String>,
    /// Maps each rendered label back to the id to emit on selection.
    pub label_to_id: HashMap<String, String>,
}

/// Extend a generated label until it is unique among the already placed ones.
///
/// The id is appended first; a numeric suffix is added only if that still
/// collides, so the result is deterministic for a given set of entries.
fn unique_label(candidate: String, id: &str, used: &mut HashSet<String>) -> String {
    if used.insert(candidate.clone()) {
        return candidate;
    }
    let qualified = format!("{candidate} ({id})");
    if used.insert(qualified.clone()) {
        return qualified;
    }
    let mut suffix = 2usize;
    loop {
        let numbered = format!("{qualified} #{suffix}");
        if used.insert(numbered.clone()) {
            return numbered;
        }
        suffix += 1;
    }
}

/// Build the labels of a lookup select from `(id, display)` entries.
///
/// Repeated display names are qualified with their id so each label maps to
/// exactly one entry. An id absent from the lookup gets a visible fallback
/// label, keeping the selection from rendering blank. Generated labels are
/// extended when they would otherwise collide with another rendered label.
pub fn lookup_select(entries: &[(String, String)], selected_id: &str) -> LookupSelect {
    let bases: Vec<String> = entries
        .iter()
        .map(|(id, display)| {
            if display.is_empty() {
                format!("Unknown ({id})")
            } else {
                display.clone()
            }
        })
        .collect();

    let mut repeats: HashMap<&str, usize> = HashMap::new();
    for base in &bases {
        *repeats.entry(base.as_str()).or_insert(0) += 1;
    }

    // Names shown verbatim are reserved first so generated labels never take
    // another entry's display.
    let mut used: HashSet<String> = HashSet::new();
    for base in &bases {
        if repeats.get(base.as_str()).copied().unwrap_or(0) == 1 {
            used.insert(base.clone());
        }
    }

    let mut options = Vec::with_capacity(entries.len() + 1);
    let mut label_to_id = HashMap::new();
    let mut selected = None;

    for ((id, _), base) in entries.iter().zip(bases.iter()) {
        let label = if repeats.get(base.as_str()).copied().unwrap_or(0) > 1 {
            unique_label(format!("{base} ({id})"), id, &mut used)
        } else {
            base.clone()
        };
        if id == selected_id {
            selected = Some(label.clone());
        }
        label_to_id.insert(label.clone(), id.clone());
        options.push(label);
    }

    if selected.is_none() {
        let fallback = unique_label(format!("Unknown ({selected_id})"), selected_id, &mut used);
        label_to_id.insert(fallback.clone(), selected_id.to_string());
        options.push(fallback.clone());
        selected = Some(fallback);
    }

    LookupSelect {
        options,
        selected,
        label_to_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_select_missing_id_gets_visible_label() {
        let entries = [("7".to_string(), "Invented Creature".to_string())];
        let select = lookup_select(&entries, "9");
        assert_eq!(select.selected.as_deref(), Some("Unknown (9)"));
        assert!(select.options.iter().any(|label| label == "Unknown (9)"));
        assert_eq!(
            select.label_to_id.get("Unknown (9)").map(String::as_str),
            Some("9")
        );
    }

    #[test]
    fn test_lookup_select_disambiguates_repeated_display_names() {
        let entries = [
            ("7".to_string(), "Invented Creature".to_string()),
            ("9".to_string(), "Invented Creature".to_string()),
        ];
        let select = lookup_select(&entries, "9");
        assert_eq!(select.selected.as_deref(), Some("Invented Creature (9)"));
        assert_eq!(
            select
                .label_to_id
                .get("Invented Creature (7)")
                .map(String::as_str),
            Some("7")
        );
        assert_eq!(
            select
                .label_to_id
                .get("Invented Creature (9)")
                .map(String::as_str),
            Some("9")
        );
    }

    #[test]
    fn test_lookup_select_blank_display_gets_visible_label() {
        let entries = [("4".to_string(), String::new())];
        let select = lookup_select(&entries, "4");
        assert_eq!(select.selected.as_deref(), Some("Unknown (4)"));
    }

    #[test]
    fn test_lookup_select_fallback_label_does_not_shadow_a_real_entry() {
        // The entry display already looks like the fallback for the missing
        // selected id; both labels must stay distinct and map to their own id.
        let entries = [("7".to_string(), "Unknown (9)".to_string())];
        let select = lookup_select(&entries, "9");

        assert_eq!(select.options.len(), 2, "the fallback must add an option");
        assert_eq!(
            select
                .options
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            2,
            "every rendered label must be distinct, got {:?}",
            select.options
        );
        let fallback = select.selected.as_deref().expect("a fallback is selected");
        assert_ne!(
            fallback, "Unknown (9)",
            "the fallback must be disambiguated"
        );
        assert_eq!(fallback, "Unknown (9) (9)");
        assert_eq!(
            select.label_to_id.get(fallback).map(String::as_str),
            Some("9"),
            "selecting the fallback must emit the missing id"
        );
        assert_eq!(
            select.label_to_id.get("Unknown (9)").map(String::as_str),
            Some("7"),
            "selecting the real entry must still emit its own id"
        );
    }

    #[test]
    fn test_lookup_select_qualified_label_does_not_shadow_a_third_entry() {
        // "Foo (7)" is both a qualified label for the repeated "Foo" base and
        // the plain display of a third entry; the generated one must move.
        let entries = [
            ("7".to_string(), "Foo".to_string()),
            ("9".to_string(), "Foo".to_string()),
            ("5".to_string(), "Foo (7)".to_string()),
        ];
        let select = lookup_select(&entries, "9");

        assert_eq!(
            select
                .options
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            select.options.len(),
            "every rendered label must be distinct, got {:?}",
            select.options
        );
        assert_eq!(select.selected.as_deref(), Some("Foo (9)"));
        assert_eq!(
            select.label_to_id.get("Foo (7) (7)").map(String::as_str),
            Some("7"),
            "the qualified label must be extended and keep its own id"
        );
        assert_eq!(
            select.label_to_id.get("Foo (9)").map(String::as_str),
            Some("9")
        );
        assert_eq!(
            select.label_to_id.get("Foo (7)").map(String::as_str),
            Some("5"),
            "the third entry's plain display must keep its own id"
        );
    }
}
