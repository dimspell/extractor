use std::collections::HashMap;

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

/// Build the labels of a lookup select from `(id, display)` entries.
///
/// Repeated display names are qualified with their id so each label maps to
/// exactly one entry. An id absent from the lookup gets a visible fallback
/// label, keeping the selection from rendering blank.
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

    let mut options = Vec::with_capacity(entries.len() + 1);
    let mut label_to_id = HashMap::new();
    let mut selected = None;

    for ((id, _), base) in entries.iter().zip(bases.iter()) {
        let label = if repeats.get(base.as_str()).copied().unwrap_or(0) > 1 {
            format!("{base} ({id})")
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
        let fallback = format!("Unknown ({selected_id})");
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
}
