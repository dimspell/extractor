//! Guards against fake 50% progress bars in loading views. Loading has no
//! measurable progress, so the views show an indeterminate "Working…" label.

const LOADING_VIEW_SOURCES: [&str; 4] = [
    include_str!("../view/mod.rs"),
    include_str!("../editors/map_editor/view/mod.rs"),
    include_str!("../view/editor/spreadsheet/view/status_bar.rs"),
    include_str!("../editors/save_file_viewer/map_preview/view.rs"),
];

#[test]
fn test_loading_views_have_no_half_progress_bar() {
    for source in LOADING_VIEW_SOURCES {
        assert!(
            !source.contains("progress_bar(0.0..=1.0, 0.5)"),
            "loading view must not render a fake 50% progress bar"
        );
        assert!(
            !source.contains("progress_bar(0.0..=100.0, 50.0)"),
            "loading view must not render a fake 50% progress bar"
        );
    }
    assert!(
        LOADING_VIEW_SOURCES
            .iter()
            .all(|source| source.contains("Working…")),
        "every loading view must show an indeterminate indicator"
    );
}
