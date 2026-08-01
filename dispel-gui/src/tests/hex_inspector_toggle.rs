//! End-to-end tests for the hex editor's inspector A/B source toggle,
//! exercising the REAL dispel-gui widget tree and message chain:
//! button click → Message::Editor → dispatch table → hex_wrapper::handle
//! → hexedit::update → re-render.

#[cfg(all(test, feature = "iced_test"))]
mod hex_inspector_toggle_tests {
    use std::cell::Cell;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    use gui_widgets::components::paragraph_cache::ParagraphCache;
    use hexedit::domain::panel::default_pane_grid;
    use hexedit::domain::provider::BufferProvider;
    use hexedit::domain::search::SearchState;
    use hexedit::domain::selection::Selection;
    use hexedit::domain::write_mode::WriteMode;
    use hexedit::ui::coloring::ColorScheme;
    use hexedit::ui::theme::{ThemeVariant, DARK_THEME};
    use hexedit::{ComparisonFile, HexEditorState, InspectorSource};
    use hexedit::{HexEditorMessage, LuaScriptEngine};
    use iced_test::simulator;

    use crate::message::editor::EditorMessage;
    use crate::message::Message;
    use crate::tests::app_with_tab;
    use crate::workspace::EditorType::HexEditor;

    fn state_with_comparison(baseline: Vec<u8>, comparison: Vec<u8>) -> HexEditorState {
        let panes = default_pane_grid();
        let pane_focus = *panes.iter().next().map(|(id, _)| id).unwrap();
        let diff = baseline
            .iter()
            .zip(comparison.iter())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| i as u64)
            .collect();
        HexEditorState {
            path: PathBuf::from("test.bin"),
            name: "test.bin".to_string(),
            panes,
            pane_focus,
            provider: BufferProvider::from_bytes(baseline),
            bytes_per_row: 16,
            selection: Selection::single(0),
            edit_mode: None,
            inspector_edit: None,
            inspector_source: InspectorSource::Baseline,
            vanilla: None,
            vanilla_diff: BTreeSet::new(),
            comparison_file: Some(ComparisonFile {
                name: "other.bin".into(),
                data: comparison,
                diff,
            }),
            diff_review: false,
            patterns: Vec::new(),
            pattern_by_addr: BTreeMap::new(),
            show_pattern_list: false,
            next_pattern_id: 0,
            groups: Vec::new(),
            next_group_id: 0,
            collapsed_groups: BTreeSet::new(),
            context_menu_addr: None,
            goto: None,
            search: SearchState::new(),
            show_decimal: false,
            status_msg: String::new(),
            error: None,
            cache: ParagraphCache::default(),
            lua_engine: LuaScriptEngine::default(),
            export_config: None,
            fill_dialog: None,
            extend_dialog: None,
            repeat_pattern: None,
            row_annotations: BTreeMap::new(),
            active_patterns: BTreeSet::new(),
            renaming_group: None,
            renaming_group_draft: String::new(),
            color_scheme: ColorScheme::Monochrome,
            dim_nulls: true,
            settings_open: false,
            write_mode: WriteMode::Hex,
            custom_encodings: Vec::new(),
            encoding_settings_open: false,
            encoding_settings_selection: None,
            show_stats: false,
            file_stats: None,
            selection_stats: None,
            row_entropies: None,
            show_entropy_band: true,
            show_minimap: true,
            theme: &DARK_THEME,
            theme_variant: ThemeVariant::Dark,
            pending_center_on: Cell::new(None),
        }
    }

    fn app_with_hex_and_comparison() -> crate::app::App {
        let mut app = app_with_tab(HexEditor);
        let tab_id = app.state.workspace.active().unwrap().id;
        app.state.editors.hex_editors.insert(
            tab_id,
            state_with_comparison(vec![0x2A, 0x00, 0x00, 0x00], vec![0x5A, 0x00, 0x00, 0x00]),
        );
        app
    }

    #[test]
    fn test_inspector_shows_baseline_value_in_real_view() {
        let app = app_with_hex_and_comparison();
        let mut ui = simulator(app.view());
        ui.find("42")
            .expect("inspector should decode baseline byte 0x2A as 42");
    }

    #[test]
    fn test_toggle_button_click_emits_source_message_in_real_view() {
        let app = app_with_hex_and_comparison();
        let mut ui = simulator(app.view());
        ui.click("B")
            .expect("B toggle button must exist in the rendered app");
        let messages: Vec<Message> = ui.into_messages().collect();
        assert!(
            messages.iter().any(|m| matches!(
                m,
                Message::Editor(EditorMessage::HexEditor(
                    HexEditorMessage::SetInspectorSource(InspectorSource::Comparison)
                ))
            )),
            "clicking B must emit SetInspectorSource(Comparison), got {messages:?}"
        );
    }

    #[test]
    fn test_toggle_via_update_chain_changes_values_in_real_view() {
        let mut app = app_with_hex_and_comparison();

        // Baseline: inspector decodes the main file's bytes.
        {
            let mut ui = simulator(app.view());
            ui.find("42").expect("baseline u8 should be 42");
        }

        // Drive the toggle through the app's full message chain.
        let task = app.update(Message::Editor(EditorMessage::HexEditor(
            HexEditorMessage::SetInspectorSource(InspectorSource::Comparison),
        )));
        assert_eq!(task.units(), 0, "source toggle is synchronous");

        // Re-render: the inspector must now decode the comparison file.
        let mut ui = simulator(app.view());
        ui.find("90")
            .expect("comparison u8 should be 90 after the toggle message");
        ui.find("42")
            .expect_err("baseline value must not be shown after switching to B");
    }

    #[test]
    fn test_source_label_switches_in_real_view() {
        let mut app = app_with_hex_and_comparison();

        // Baseline label: main file name.
        {
            let mut ui = simulator(app.view());
            ui.find("· test.bin")
                .expect("label should show the main file name");
        }

        // Toggle to comparison: label must switch to the other file.
        app.update(Message::Editor(EditorMessage::HexEditor(
            HexEditorMessage::SetInspectorSource(InspectorSource::Comparison),
        )));
        let mut ui = simulator(app.view());
        ui.find("· other.bin")
            .expect("label should show the comparison file name");
        ui.find("· test.bin")
            .expect_err("baseline label must be gone");
    }

    #[test]
    fn test_full_loop_click_toggle_then_rerender() {
        // The complete user flow in one test: click the real "B" button in
        // the rendered app, route the emitted message through App::update,
        // then re-render and verify the source label and values switched.
        let mut app = app_with_hex_and_comparison();

        // Baseline state.
        {
            let mut ui = simulator(app.view());
            ui.find("· test.bin").expect("baseline label before click");
            ui.find("42").expect("baseline value before click");
        }

        // Click the real button and collect what the app would receive.
        let mut ui = simulator(app.view());
        {
            use iced_test::selector::Candidate;
            ui.find(|c: Candidate| -> Option<()> {
                if let Some(vb) = c.visible_bounds() {
                    if vb.y <= 235.0 && vb.y + vb.height >= 205.0 {
                        match &c {
                            Candidate::Text { content, .. } => {
                                eprintln!("MATRIX DEBUG text {content:?} bounds={vb:?}");
                            }
                            Candidate::Container { .. } => {
                                eprintln!("MATRIX DEBUG container bounds={vb:?}");
                            }
                            Candidate::Focusable { .. } => {
                                eprintln!("MATRIX DEBUG focusable bounds={vb:?}");
                            }
                            _ => {
                                eprintln!("MATRIX DEBUG other bounds={vb:?}");
                            }
                        }
                    }
                }
                None
            });
        }
        ui.click("B").expect("B toggle button must be clickable");
        let messages: Vec<Message> = ui.into_messages().collect();
        assert!(
            !messages.is_empty(),
            "clicking B must produce at least one message"
        );

        // Route every emitted message through the app's update chain.
        for msg in messages {
            let _ = app.update(msg);
        }

        // Re-render: the label and the decoded values must have switched.
        let mut ui = simulator(app.view());
        ui.find("· other.bin")
            .expect("label must switch to the comparison file after the click");
        ui.find("· test.bin")
            .expect_err("baseline label must be gone");
        ui.find("90")
            .expect("inspector must decode the comparison byte after the click");
        ui.find("42")
            .expect_err("baseline value must be gone after the click");
    }

    #[test]
    fn test_toggle_click_works_with_diff_view_active() {
        // Exact user repro: the editor is in DIFF view (focused pane shows
        // the comparison), cursor at address 0 where baseline = 0x2A (42)
        // and comparison = 0x5A (90). Clicking the "B" toggle must switch
        // the inspector value from 42 to 90.
        let mut app = app_with_hex_and_comparison();

        // Switch the focused pane to Diff, exactly like ComparisonFileLoaded does.
        let tab_id = app.state.workspace.active().unwrap().id;
        {
            let state = app.state.editors.hex_editors.get_mut(&tab_id).unwrap();
            let focus = state.pane_focus;
            if let Some(panel) = state.panes.get_mut(focus) {
                panel.content = hexedit::HexPanelContent::Diff;
            }
        }

        // Sanity: baseline value and label before the click.
        {
            let mut ui = simulator(app.view());
            ui.find("· test.bin")
                .expect("label should show the main file");
            ui.find("42")
                .expect("inspector should decode baseline byte 0x2A");
        }

        // Click the real "B" toggle button and route through the app.
        let mut ui = simulator(app.view());
        // Debug: pane layout of the hex editor state.
        {
            let tab_id = app.state.workspace.active().unwrap().id;
            let st = app.state.editors.hex_editors.get(&tab_id).unwrap();
            eprintln!("DEBUG-DIFF pane_focus={:?}", st.pane_focus);
            for (id, panel) in st.panes.iter() {
                eprintln!("DEBUG-DIFF pane id={id:?} content={:?}", panel.content);
            }
            let b = ui.find("B").expect("B node");
            let center = b.visible_bounds().unwrap().center();
            eprintln!("DEBUG-DIFF point={center:?}");
            let hit = ui.find(center).expect("widget at point");
            eprintln!("DEBUG-DIFF top widget at point: {hit:?}");
        }
        ui.click("B")
            .expect("B toggle must be clickable with the diff view active");
        let messages: Vec<Message> = ui.into_messages().collect();
        eprintln!("DEBUG diff-view-active click messages: {messages:?}");
        for msg in messages {
            let _ = app.update(msg);
        }

        // The value must switch to the comparison file's byte.
        let mut ui = simulator(app.view());
        ui.find("· other.bin")
            .expect("label must switch to the comparison file in diff view");
        ui.find("90")
            .expect("inspector must decode the comparison byte 0x5A with the diff view active");
        ui.find("42")
            .expect_err("baseline value must be gone after the click");
    }

    #[test]
    fn test_toggle_click_toggles_back_and_forth() {
        // B → A → B: each real button click must flip the decoded source.
        let mut app = app_with_hex_and_comparison();

        let click_and_update = |app: &mut crate::app::App, label: &str| {
            let mut ui = simulator(app.view());
            ui.click(label).expect("toggle button must be clickable");
            let messages: Vec<Message> = ui.into_messages().collect();
            for msg in messages {
                let _ = app.update(msg);
            }
        };

        // Baseline first.
        {
            let mut ui = simulator(app.view());
            ui.find("42").expect("starts on baseline");
        }

        // Click B → comparison.
        click_and_update(&mut app, "B");
        {
            let mut ui = simulator(app.view());
            ui.find("90").expect("B shows the comparison byte");
            ui.find("42")
                .expect_err("baseline must be gone after clicking B");
        }

        // Click A → back to baseline.
        click_and_update(&mut app, "A");
        {
            let mut ui = simulator(app.view());
            ui.find("42").expect("A returns to the baseline byte");
            ui.find("90")
                .expect_err("comparison must be gone after clicking A");
        }

        // Click B again → comparison again.
        click_and_update(&mut app, "B");
        {
            let mut ui = simulator(app.view());
            ui.find("90")
                .expect("B switches back to the comparison byte");
        }
    }
}
