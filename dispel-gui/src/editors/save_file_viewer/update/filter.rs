use iced::Task;

use crate::components::filter::ColumnFilterOption;
use crate::editors::save_file_viewer::message::{TableFilterAction, TableKey};
use crate::editors::save_file_viewer::state::{SaveFileViewerState, TableFilterState};
use crate::message::Message;

use super::table::{
    character_table_data, events_table_data, inventory_table_data, journal_table_data,
    maps_table_data,
};

/// Numeric-aware cell comparison for sorting. Falls back to lexicographic
/// string comparison when either value is not a parseable float.
pub fn compare_cells(
    rows: &[Vec<String>],
    a: usize,
    b: usize,
    col: usize,
    ascending: bool,
) -> std::cmp::Ordering {
    let av = rows.get(a).and_then(|r| r.get(col));
    let bv = rows.get(b).and_then(|r| r.get(col));
    let ord = match (av, bv) {
        (Some(a), Some(b)) => match (a.parse::<f64>(), b.parse::<f64>()) {
            (Ok(an), Ok(bn)) => an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal),
            _ => a.cmp(b),
        },
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    };
    if ascending { ord } else { ord.reverse() }
}

/// Row height used by every save-file-viewer table (kept in sync with the
/// `TableWidget::new` `row_height` argument in the view files). Used to scroll
/// a highlighted row into view during Highlight-mode navigation.
const FILTER_ROW_HEIGHT: f32 = 22.0;

/// Dispatches a unified column-filter action to the table identified by `key`.
pub fn handle_table_filter(
    state: &mut SaveFileViewerState,
    key: TableKey,
    action: TableFilterAction,
) -> Task<Message> {
    match action {
        TableFilterAction::NextHighlight => return navigate_highlight(state, key, true),
        TableFilterAction::PrevHighlight => return navigate_highlight(state, key, false),
        _ => {}
    }

    let Some((filter, rows, indices)) = table_filter_access(state, key) else {
        return Task::none();
    };

    match action {
        TableFilterAction::OpenColumnFilter(col) => {
            filter.active_column_filter = Some(col);
            filter.column_filter_search.clear();
            filter.column_filter_options = unique_values(rows, col);
        }
        TableFilterAction::ToggleColumnFilterValue(col, value) => {
            let set = filter.column_filters.entry(col).or_default();
            if !set.insert(value.clone()) {
                set.remove(&value);
            }
            if set.is_empty() {
                filter.column_filters.remove(&col);
            }
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::SelectAllColumnFilter(col) => {
            let search = filter.column_filter_search.to_lowercase();
            let values: std::collections::HashSet<String> = filter
                .column_filter_options
                .iter()
                .filter(|o| o.value.to_lowercase().contains(&search))
                .map(|o| o.value.clone())
                .collect();
            filter.column_filters.insert(col, values);
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::ClearAllColumnFilter(col) => {
            let search = filter.column_filter_search.to_lowercase();
            let remove: std::collections::HashSet<String> = filter
                .column_filter_options
                .iter()
                .filter(|o| o.value.to_lowercase().contains(&search))
                .map(|o| o.value.clone())
                .collect();
            let current = filter.column_filters.entry(col).or_default();
            *current = current.difference(&remove).cloned().collect();
            if current.is_empty() {
                filter.column_filters.remove(&col);
            }
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::ColumnFilterSearch(s) => {
            filter.column_filter_search = s;
        }
        TableFilterAction::CloseColumnFilterModal => {
            filter.active_column_filter = None;
        }
        TableFilterAction::ClearColumnFilter(col) => {
            filter.column_filters.remove(&col);
            filter.active_column_filter = None;
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::QuickFilter(col, value) => {
            let mut set = std::collections::HashSet::new();
            set.insert(value);
            filter.column_filters.insert(col, set);
            filter.active_column_filter = None;
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::QueryChanged(s) => {
            filter.filter_query = s;
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::SetMode(mode) => {
            filter.filter_mode = mode;
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::ClearAllFilters => {
            filter.column_filters.clear();
            filter.filter_query.clear();
            filter.active_column_filter = None;
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::NextHighlight | TableFilterAction::PrevHighlight => {}
    }
    Task::none()
}

/// Borrow the filter state alongside the (immutable) display rows and the
/// (mutable) filtered indices for the table identified by `key`. The filter
/// state and the caches live in disjoint fields of `SaveFileViewerState`, so
/// both can be mutably borrowed at once.
#[allow(clippy::type_complexity)]
fn table_filter_access(
    state: &mut SaveFileViewerState,
    key: TableKey,
) -> Option<(&mut TableFilterState, &[Vec<String>], &mut Vec<usize>)> {
    match key {
        TableKey::Map(map, kind) => {
            let ts = state.maps_table_states.get_mut(map)?.get_mut(&kind)?;
            let filter = &mut ts.filter;
            let (rows, indices) = maps_table_data(&mut state.maps_display_caches[map], kind);
            Some((filter, rows, indices))
        }
        TableKey::Inventory(cat) => {
            let ts = state.inventory_table_states.get_mut(&cat)?;
            let filter = &mut ts.filter;
            let (rows, indices) = inventory_table_data(
                &mut state.inventory_display_caches,
                &mut state.inventory_filtered_indices,
                cat,
            );
            Some((filter, rows, indices))
        }
        TableKey::Character(kind) => {
            let ts = state.character_table_states.get_mut(&kind)?;
            let filter = &mut ts.filter;
            let (rows, indices) = character_table_data(
                &mut state.character_display_caches,
                &mut state.character_filtered_indices,
                kind,
            );
            Some((filter, rows, indices))
        }
        TableKey::Events => {
            let filter = &mut state.events_table_state.filter;
            let (rows, indices) = events_table_data(
                &mut state.events_display_cache,
                &mut state.events_filtered_indices,
            );
            Some((filter, rows, indices))
        }
        TableKey::Journal(section) => {
            let ts = state.journal_table_states.get_mut(&section)?;
            let filter = &mut ts.filter;
            let (rows, indices) = journal_table_data(
                &mut state.journal_display_caches,
                &mut state.journal_filtered_indices,
                section,
            );
            Some((filter, rows, indices))
        }
    }
}

/// Rebuild `filtered_indices` / `highlighted_indices` from the current
/// `filter_query`, `filter_mode`, and `column_filters`. Mirrors the
/// spreadsheet editor's `apply_filter`.
fn apply_table_filter(
    rows: &[Vec<String>],
    filter: &mut TableFilterState,
    indices: &mut Vec<usize>,
) {
    use crate::components::filter::GlobalFilterMode;

    filter.highlighted_indices.clear();

    let has_query = !filter.filter_query.is_empty();
    let has_col = !filter.column_filters.is_empty();

    let col_matches = |row: &[String]| -> bool {
        for (&col, selected) in &filter.column_filters {
            if let Some(value) = row.get(col)
                && !selected.is_empty()
                && !selected.contains(value)
            {
                return false;
            }
        }
        true
    };

    if !has_query && !has_col {
        *indices = (0..rows.len()).collect();
        return;
    }

    let query = filter.filter_query.to_lowercase();
    let matches_query =
        |row: &[String]| -> bool { row.iter().any(|cell| cell.to_lowercase().contains(&query)) };

    match filter.filter_mode {
        GlobalFilterMode::FilterOut => {
            indices.clear();
            for (idx, row) in rows.iter().enumerate() {
                let col_ok = !has_col || col_matches(row);
                let q_ok = !has_query || matches_query(row);
                if col_ok && q_ok {
                    indices.push(idx);
                }
            }
        }
        GlobalFilterMode::Highlight => {
            // Column filters hard-filter; global query only highlights.
            indices.clear();
            for (idx, row) in rows.iter().enumerate() {
                if !has_col || col_matches(row) {
                    indices.push(idx);
                    if has_query && matches_query(row) {
                        filter.highlighted_indices.push(idx);
                    }
                }
            }
            if !filter.highlighted_indices.is_empty() {
                filter.current_highlight_pos = Some(0);
            }
        }
    }
}

/// Distinct values (with counts) for a column, sorted by value.
fn unique_values(rows: &[Vec<String>], col: usize) -> Vec<ColumnFilterOption> {
    use std::collections::HashMap;

    let mut counts: HashMap<&str, usize> = HashMap::new();
    for row in rows {
        if let Some(v) = row.get(col) {
            *counts.entry(v.as_str()).or_insert(0) += 1;
        }
    }
    let mut opts: Vec<ColumnFilterOption> = counts
        .into_iter()
        .map(|(v, count)| ColumnFilterOption {
            value: v.to_string(),
            count,
        })
        .collect();
    opts.sort_by(|a, b| a.value.cmp(&b.value));
    opts
}

/// Return the visible (filtered) indices for the table identified by `key`,
/// used to translate an original index to a visible position for scrolling.
fn filtered_indices_for(state: &SaveFileViewerState, key: TableKey) -> Option<&[usize]> {
    use super::table::maps_table_indices;
    match key {
        TableKey::Map(map, kind) => state
            .maps_display_caches
            .get(map)
            .map(|c| maps_table_indices(c, kind)),
        TableKey::Inventory(cat) => state.inventory_filtered_indices.get(&cat).map(|v| &v[..]),
        TableKey::Character(kind) => state.character_filtered_indices.get(&kind).map(|v| &v[..]),
        TableKey::Events => Some(&state.events_filtered_indices),
        TableKey::Journal(section) => state.journal_filtered_indices.get(&section).map(|v| &v[..]),
    }
}

/// Step the Highlight-mode highlight cursor and bring the focused row into
/// view, mirroring the spreadsheet editor's `Navigate{Next,Prev}Highlight`.
fn navigate_highlight(state: &mut SaveFileViewerState, key: TableKey, next: bool) -> Task<Message> {
    // Advance the cursor on the table's filter state.
    match key {
        TableKey::Map(map, kind) => {
            let Some(ts) = state
                .maps_table_states
                .get_mut(map)
                .and_then(|m| m.get_mut(&kind))
            else {
                return Task::none();
            };
            if next {
                ts.filter.navigate_next_highlight();
            } else {
                ts.filter.navigate_prev_highlight();
            }
        }
        TableKey::Inventory(cat) => {
            let Some(ts) = state.inventory_table_states.get_mut(&cat) else {
                return Task::none();
            };
            if next {
                ts.filter.navigate_next_highlight();
            } else {
                ts.filter.navigate_prev_highlight();
            }
        }
        TableKey::Character(kind) => {
            let Some(ts) = state.character_table_states.get_mut(&kind) else {
                return Task::none();
            };
            if next {
                ts.filter.navigate_next_highlight();
            } else {
                ts.filter.navigate_prev_highlight();
            }
        }
        TableKey::Events => {
            if next {
                state.events_table_state.filter.navigate_next_highlight();
            } else {
                state.events_table_state.filter.navigate_prev_highlight();
            }
        }
        TableKey::Journal(section) => {
            let Some(ts) = state.journal_table_states.get_mut(&section) else {
                return Task::none();
            };
            if next {
                ts.filter.navigate_next_highlight();
            } else {
                ts.filter.navigate_prev_highlight();
            }
        }
    }

    // Resolve the focused original index, then the focused table state so we
    // can update selection + scroll. We re-fetch the table state here because
    // the filter state above lives inside it.
    let orig = match key {
        TableKey::Map(map, kind) => state
            .maps_table_states
            .get(map)
            .and_then(|m| m.get(&kind))
            .and_then(|ts| ts.filter.current_highlight_orig_idx()),
        TableKey::Inventory(cat) => state
            .inventory_table_states
            .get(&cat)
            .and_then(|ts| ts.filter.current_highlight_orig_idx()),
        TableKey::Character(kind) => state
            .character_table_states
            .get(&kind)
            .and_then(|ts| ts.filter.current_highlight_orig_idx()),
        TableKey::Events => state.events_table_state.filter.current_highlight_orig_idx(),
        TableKey::Journal(section) => state
            .journal_table_states
            .get(&section)
            .and_then(|ts| ts.filter.current_highlight_orig_idx()),
    };

    let Some(orig) = orig else {
        return Task::none();
    };

    let visible =
        filtered_indices_for(state, key).and_then(|idxs| idxs.iter().position(|&i| i == orig));

    match (key, visible) {
        (TableKey::Map(map, kind), Some(fidx)) => {
            if let Some(ts) = state
                .maps_table_states
                .get_mut(map)
                .and_then(|m| m.get_mut(&kind))
            {
                ts.selected_orig = Some(orig);
                ts.table_state.scroll_offset.y = fidx as f32 * FILTER_ROW_HEIGHT;
            }
        }
        (TableKey::Inventory(cat), Some(fidx)) => {
            if let Some(ts) = state.inventory_table_states.get_mut(&cat) {
                ts.selected_orig = Some(orig);
                ts.table_state.scroll_offset.y = fidx as f32 * FILTER_ROW_HEIGHT;
            }
        }
        (TableKey::Character(kind), Some(fidx)) => {
            if let Some(ts) = state.character_table_states.get_mut(&kind) {
                ts.selected_orig = Some(orig);
                ts.table_state.scroll_offset.y = fidx as f32 * FILTER_ROW_HEIGHT;
            }
        }
        (TableKey::Events, Some(fidx)) => {
            state.events_table_state.selected_orig = Some(orig);
            state.events_table_state.table_state.scroll_offset.y = fidx as f32 * FILTER_ROW_HEIGHT;
        }
        (TableKey::Journal(section), Some(fidx)) => {
            if let Some(ts) = state.journal_table_states.get_mut(&section) {
                ts.selected_orig = Some(orig);
                ts.table_state.scroll_offset.y = fidx as f32 * FILTER_ROW_HEIGHT;
            }
        }
        _ => {}
    }
    Task::none()
}

#[cfg(test)]
mod tests {
    use super::compare_cells;

    #[test]
    fn compare_cells_numeric_ascending() {
        let rows = vec![
            vec!["10".to_string(), "a".to_string()],
            vec!["2".to_string(), "b".to_string()],
            vec!["5".to_string(), "c".to_string()],
        ];
        let mut indices: Vec<usize> = vec![0, 1, 2];
        indices.sort_by(|&a, &b| compare_cells(&rows, a, b, 0, true));
        assert_eq!(indices, vec![1, 2, 0]); // 2, 5, 10
    }

    #[test]
    fn compare_cells_numeric_descending() {
        let rows = vec![
            vec!["10".to_string(), "a".to_string()],
            vec!["2".to_string(), "b".to_string()],
            vec!["5".to_string(), "c".to_string()],
        ];
        let mut indices: Vec<usize> = vec![0, 1, 2];
        indices.sort_by(|&a, &b| compare_cells(&rows, a, b, 0, false));
        assert_eq!(indices, vec![0, 2, 1]); // 10, 5, 2
    }

    #[test]
    fn compare_cells_lexicographic() {
        let rows = vec![
            vec!["apple".to_string()],
            vec!["banana".to_string()],
            vec!["cherry".to_string()],
        ];
        let mut indices: Vec<usize> = vec![0, 1, 2];
        indices.sort_by(|&a, &b| compare_cells(&rows, a, b, 0, true));
        assert_eq!(indices, vec![0, 1, 2]);
    }

    #[test]
    fn compare_cells_mixed() {
        let rows = vec![
            vec!["42".to_string()],
            vec!["hello".to_string()],
            vec!["7".to_string()],
        ];
        let mut indices: Vec<usize> = vec![0, 1, 2];
        indices.sort_by(|&a, &b| compare_cells(&rows, a, b, 0, true));
        // 7 < 42 < "hello" (lexicographic after numerics)
        assert_eq!(indices, vec![2, 0, 1]);
    }

    #[test]
    fn compare_cells_missing_column() {
        let rows = vec![vec!["a".to_string()], vec!["b".to_string()]];
        let mut indices: Vec<usize> = vec![0, 1];
        indices.sort_by(|&a, &b| compare_cells(&rows, a, b, 1, true));
        // Both None → Equal, stable sort preserves original order
        assert_eq!(indices, vec![0, 1]);
    }
}
