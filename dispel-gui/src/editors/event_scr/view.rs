use std::sync::atomic::Ordering;

use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::components::utils::horizontal_rule as hr;
use crate::editors::event_scr::message::EventScrEditorMessage;
use crate::editors::event_scr::state::{EventScriptEditorState, FunctionIndexState, SectionTab};
use crate::style;
use iced::widget::{
    button, column, container, progress_bar, row, scrollable, text, text_input, Space,
};
use iced::{font, Alignment, Border, Color, Element, Font, Length};

pub fn view(app: &App) -> Element<'_, EventScrEditorMessage> {
    let state = &app.state.event_scr_editor;

    let base = match &state.script_loading {
        LoadingState::Loaded(script) => {
            let script_id = script.id;
            let active_tab = state.active_section;

            // Tab bar
            let tabs = row![
                tab_button(SectionTab::Header, active_tab),
                tab_button(SectionTab::Var, active_tab),
                tab_button(SectionTab::Map, active_tab),
                tab_button(SectionTab::Chr, active_tab),
                tab_button(SectionTab::Npc, active_tab),
                tab_button(SectionTab::Spr, active_tab),
                tab_button(SectionTab::Wav, active_tab),
                tab_button(SectionTab::Act, active_tab),
            ]
            .spacing(5)
            .wrap();

            // Section content
            let content: Element<EventScrEditorMessage> = match active_tab {
                SectionTab::Header => view_header(script),
                SectionTab::Var => view_var_section(script),
                SectionTab::Map => view_line_section(script, SectionTab::Map),
                SectionTab::Chr => view_line_section(script, SectionTab::Chr),
                SectionTab::Npc => view_line_section(script, SectionTab::Npc),
                SectionTab::Spr => view_spr_section(script),
                SectionTab::Wav => view_line_section(script, SectionTab::Wav),
                SectionTab::Act => view_act_section(script, state),
            };

            let modified_indicator = if state.modified { " ●" } else { "" };

            // Save button and errors
            let save_button = button("Save")
                .on_press(EventScrEditorMessage::SaveScript)
                .style(if state.modified {
                    button::primary
                } else {
                    button::secondary
                });

            let save_error: Element<EventScrEditorMessage> = if let Some(ref err) = state.save_error
            {
                text(err)
                    .size(14)
                    .color(iced::Color::from_rgb(1.0, 0.0, 0.0))
                    .into()
            } else {
                text("").into()
            };

            column![
                row![
                    text(format!("EventScript [{}]", script_id)).size(20),
                    text(modified_indicator)
                        .size(20)
                        .color(iced::Color::from_rgb(1.0, 0.8, 0.0)),
                    Space::new().width(Length::Fill),
                    save_button,
                ]
                .align_y(iced::Alignment::Center),
                tabs,
                content,
                save_error,
                hr(1),
                view_status_bar(state),
            ]
            .spacing(10)
            .into()
        }
        LoadingState::Failed(err) => container(column![
            text("Failed to load EventScript")
                .size(20)
                .color(iced::Color::from_rgb(1.0, 0.0, 0.0)),
            text(err).size(14),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into(),
        LoadingState::Idle | LoadingState::Loading => empty_editor(),
    };

    // Wrap in indexing progress modal when scanning is active.
    if matches!(state.index_state, FunctionIndexState::Indexing { .. }) {
        let modal_content = index_progress_modal(state);
        crate::components::modal::modal(
            base,
            modal_content,
            || EventScrEditorMessage::CancelIndexing,
            0.5,
        )
    } else {
        base
    }
}

fn tab_button(tab: SectionTab, active: SectionTab) -> Element<'static, EventScrEditorMessage> {
    let is_active = tab == active;
    let label = if is_active {
        text(tab.label()).font(Font {
            weight: iced::font::Weight::Bold,
            ..Font::DEFAULT
        })
    } else {
        text(tab.label())
    };
    let btn = button(label)
        .on_press(EventScrEditorMessage::SectionChanged(tab))
        .style(if is_active {
            button::primary
        } else {
            button::secondary
        });
    btn.into()
}

fn view_header(
    script: &dispel_core::references::event_scr::EventScript,
) -> Element<'static, EventScrEditorMessage> {
    let comments: Vec<Element<EventScrEditorMessage>> = script
        .header_comments
        .iter()
        .map(|line| text(line.clone()).into())
        .collect();

    column![
        text("Header Comments").size(16).font(Font {
            weight: font::Weight::Bold,
            ..Font::DEFAULT
        }),
        column(comments).spacing(5),
    ]
    .spacing(10)
    .into()
}

fn view_var_section(
    script: &dispel_core::references::event_scr::EventScript,
) -> Element<'static, EventScrEditorMessage> {
    let header = row![
        text("Name")
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .width(Length::FillPortion(2)),
        text("Value")
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .width(Length::FillPortion(2)),
        text("Actions")
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .width(Length::FillPortion(1)),
    ]
    .spacing(10);

    let rows: Vec<Element<EventScrEditorMessage>> = script
        .variables
        .iter()
        .enumerate()
        .map(|(i, var)| {
            row![
                text_input("", &var.name)
                    .on_input(move |s| EventScrEditorMessage::VariableNameChanged(i, s))
                    .width(Length::FillPortion(2)),
                text_input("", &var.value)
                    .on_input(move |s| EventScrEditorMessage::VariableValueChanged(i, s))
                    .width(Length::FillPortion(2)),
                button("Delete")
                    .on_press(EventScrEditorMessage::VariableDeleted(i))
                    .width(Length::FillPortion(1)),
            ]
            .spacing(10)
            .into()
        })
        .collect();

    column![
        text("Variables").size(16).font(Font {
            weight: font::Weight::Bold,
            ..Font::DEFAULT
        }),
        header,
        scrollable(column(rows).spacing(5)).height(Length::Fill),
        button("+ Add Variable").on_press(EventScrEditorMessage::VariableAdded),
    ]
    .spacing(10)
    .into()
}

fn view_line_section(
    script: &dispel_core::references::event_scr::EventScript,
    section: SectionTab,
) -> Element<'static, EventScrEditorMessage> {
    let lines: &Vec<String> = match section {
        SectionTab::Map => &script.map_content,
        SectionTab::Chr => &script.chr_content,
        SectionTab::Npc => &script.npc_content,
        SectionTab::Wav => &script.wav_content,
        _ => return text("Invalid section").into(),
    };

    let rows: Vec<Element<EventScrEditorMessage>> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            row![
                text_input("", line)
                    .on_input(move |s| EventScrEditorMessage::LineContentChanged(section, i, s))
                    .width(Length::FillPortion(5)),
                button("Delete")
                    .on_press(EventScrEditorMessage::LineDeleted(section, i))
                    .width(Length::FillPortion(1)),
            ]
            .spacing(10)
            .into()
        })
        .collect();

    column![
        text(section.label()).size(16).font(Font {
            weight: font::Weight::Bold,
            ..Font::DEFAULT
        }),
        scrollable(column(rows).spacing(5)).height(Length::Fill),
        button("+ Add Line").on_press(EventScrEditorMessage::LineAdded(section)),
    ]
    .spacing(10)
    .into()
}

fn view_spr_section(
    script: &dispel_core::references::event_scr::EventScript,
) -> Element<'static, EventScrEditorMessage> {
    let header = row![
        text("Alias")
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .width(Length::FillPortion(2)),
        text("Filename")
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .width(Length::FillPortion(2)),
        text("Actions")
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .width(Length::FillPortion(1)),
    ]
    .spacing(10);

    let rows: Vec<Element<EventScrEditorMessage>> = script
        .spr_content
        .iter()
        .enumerate()
        .map(|(i, spr)| {
            row![
                text_input("", &spr.sprite_alias)
                    .on_input(move |s| EventScrEditorMessage::SpriteAliasChanged(i, s))
                    .width(Length::FillPortion(2)),
                text_input("", &spr.sprite_file)
                    .on_input(move |s| EventScrEditorMessage::SpriteFileChanged(i, s))
                    .width(Length::FillPortion(2)),
                button("Delete")
                    .on_press(EventScrEditorMessage::SpriteDeleted(i))
                    .width(Length::FillPortion(1)),
            ]
            .spacing(10)
            .into()
        })
        .collect();

    column![
        text("Sprites").size(16).font(Font {
            weight: font::Weight::Bold,
            ..Font::DEFAULT
        }),
        header,
        scrollable(column(rows).spacing(5)).height(Length::Fill),
        button("+ Add Sprite").on_press(EventScrEditorMessage::SpriteAdded),
    ]
    .spacing(10)
    .into()
}

fn view_act_section<'a>(
    script: &'a dispel_core::references::event_scr::EventScript,
    state: &'a EventScriptEditorState,
) -> Element<'a, EventScrEditorMessage> {
    // ── Index status / picker toolbar ────────────────────────────────────
    let index_info: Element<'a, EventScrEditorMessage> = match &state.index_state {
        FunctionIndexState::Loaded(index) => {
            let count = index.functions.len();
            row![
                text(format!("{} function(s) indexed", count))
                    .size(12)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
                button("Refresh Index")
                    .on_press(EventScrEditorMessage::BuildFunctionIndex)
                    .padding([4, 10]),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
        }
        FunctionIndexState::Failed(e) => row![
            text("Index: ")
                .size(12)
                .color(Color::from_rgb(0.8, 0.3, 0.3)),
            text(e).size(12).color(Color::from_rgb(0.8, 0.3, 0.3)),
            button("Retry")
                .on_press(EventScrEditorMessage::BuildFunctionIndex)
                .padding([4, 10]),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into(),
        _ => button("Build Index")
            .on_press(EventScrEditorMessage::BuildFunctionIndex)
            .padding([4, 10])
            .into(),
    };

    // ── Function picker ──────────────────────────────────────────────────
    let picker: Option<Element<'static, EventScrEditorMessage>> = if state.picker_open {
        Some(view_function_picker(state))
    } else {
        None
    };

    // ── Action rows ──────────────────────────────────────────────────────
    let header = row![
        text("Type")
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .width(Length::FillPortion(1)),
        text("Prefix")
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .width(Length::FillPortion(1)),
        text("Function/Content")
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .width(Length::FillPortion(3)),
        text("Params")
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .width(Length::FillPortion(2)),
        text("Actions")
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .width(Length::FillPortion(1)),
    ]
    .spacing(10);

    let rows: Vec<Element<EventScrEditorMessage>> = script
        .actions
        .iter()
        .enumerate()
        .map(|(i, act)| {
            if let Some(ref raw) = act.raw_content {
                // Raw control flow content (if/{/} etc.) - editable
                row![
                    text("Ctrl").width(Length::FillPortion(1)),
                    text("-").width(Length::FillPortion(1)),
                    text_input("", raw)
                        .on_input(move |s| EventScrEditorMessage::ActionRawContentChanged(i, s))
                        .width(Length::FillPortion(3)),
                    text("-").width(Length::FillPortion(2)),
                    button("Delete")
                        .on_press(EventScrEditorMessage::ActionDeleted(i))
                        .width(Length::FillPortion(1)),
                ]
                .spacing(10)
                .into()
            } else {
                // Structured action function
                let params_str = act.parameters.join(", ");
                row![
                    text("Func").width(Length::FillPortion(1)),
                    text_input("", act.prefix.as_deref().unwrap_or(""))
                        .on_input(move |s| EventScrEditorMessage::ActionPrefixChanged(i, s))
                        .width(Length::FillPortion(1)),
                    text_input("", &act.function_name)
                        .on_input(move |s| EventScrEditorMessage::ActionFunctionChanged(i, s))
                        .width(Length::FillPortion(3)),
                    text_input("", &params_str)
                        .on_input(move |s| EventScrEditorMessage::ActionParamsChanged(i, s))
                        .width(Length::FillPortion(2)),
                    button("Delete")
                        .on_press(EventScrEditorMessage::ActionDeleted(i))
                        .width(Length::FillPortion(1)),
                ]
                .spacing(10)
                .into()
            }
        })
        .collect();

    let mut act_content: Vec<Element<EventScrEditorMessage>> = Vec::new();

    act_content.push(
        text("Action Functions")
            .size(16)
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::DEFAULT
            })
            .into(),
    );

    act_content.push(index_info);

    if let Some(picker_view) = picker {
        act_content.push(picker_view);
    }

    act_content.push(
        row![
            button("+ Add Action").on_press(EventScrEditorMessage::ActionAdded),
            button("+ Add Raw Text").on_press(EventScrEditorMessage::ActionRawAdded),
            button("Pick Function").on_press(EventScrEditorMessage::ToggleFunctionPicker),
        ]
        .spacing(10)
        .into(),
    );

    act_content.push(header.into());
    act_content.push(
        scrollable(column(rows).spacing(5))
            .height(Length::Fill)
            .into(),
    );

    column(act_content).spacing(10).into()
}

fn view_status_bar(state: &EventScriptEditorState) -> Element<'static, EventScrEditorMessage> {
    let file_path = state
        .file_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "No file loaded".to_string());

    let encoding_badge = container(text("EUC-KR").size(12))
        .padding([2, 6])
        .style(|_theme| container::Style {
            background: Some(Color::from_rgb(0.2, 0.6, 0.2).into()),
            border: Border::default(),
            text_color: Some(Color::WHITE),
            ..Default::default()
        });

    let modified_text = if state.modified {
        text("Modified")
            .size(12)
            .color(Color::from_rgb(1.0, 0.8, 0.0))
    } else {
        text("Saved").size(12).color(Color::from_rgb(0.5, 0.5, 0.5))
    };

    let error_count = state.act_parse_errors.len();
    let error_text = if error_count > 0 {
        text(format!("{} error(s)", error_count))
            .size(12)
            .color(Color::from_rgb(1.0, 0.0, 0.0))
    } else {
        text("No errors")
            .size(12)
            .color(Color::from_rgb(0.5, 0.5, 0.5))
    };

    let shortcut_text = text("Ctrl+S to save")
        .size(11)
        .color(Color::from_rgb(0.6, 0.6, 0.6));

    row![
        text(file_path)
            .size(12)
            .color(Color::from_rgb(0.7, 0.7, 0.7))
            .width(Length::Fill),
        encoding_badge,
        separator(),
        modified_text,
        separator(),
        error_text,
        separator(),
        shortcut_text,
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

fn index_progress_modal(state: &EventScriptEditorState) -> Element<'static, EventScrEditorMessage> {
    let (processed, total, current_file, _cancelled) =
        if let FunctionIndexState::Indexing { ref progress } = state.index_state {
            (
                progress.processed.load(Ordering::Relaxed),
                progress.total.load(Ordering::Relaxed),
                progress.current_file.lock().unwrap().clone(),
                progress.cancelled.load(Ordering::Relaxed),
            )
        } else {
            (0, 1, String::new(), true)
        };

    let fraction = if total > 0 {
        processed as f32 / total as f32
    } else {
        0.0
    };
    let pct = (fraction * 100.0) as u32;

    let content = column![
        text("Indexing Event Scripts").size(16),
        hr(1),
        progress_bar(0.0..=1.0, fraction),
        text(format!("{}% — {} / {} files", pct, processed, total)).size(13),
        text(current_file)
            .size(11)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
        hr(1),
        button(text("Cancel").size(13))
            .on_press(EventScrEditorMessage::CancelIndexing)
            .padding([6, 20]),
    ]
    .spacing(12)
    .padding(24)
    .width(360);

    container(content).style(style::modal_container).into()
}

fn view_function_picker(state: &EventScriptEditorState) -> Element<'static, EventScrEditorMessage> {
    let filter = &state.picker_filter;
    let functions: Vec<&crate::editors::event_scr::functions::IndexedFunction> = match &state
        .index_state
    {
        FunctionIndexState::Loaded(index) => index
            .functions
            .iter()
            .filter(|f| filter.is_empty() || f.name.to_lowercase().contains(&filter.to_lowercase()))
            .collect(),
        _ => Vec::new(),
    };

    let list: Vec<Element<EventScrEditorMessage>> = functions
        .iter()
        .map(|f| {
            let label = format!(
                "{} ({} param{}) — {}×",
                f.name,
                f.param_count,
                if f.param_count == 1 { "" } else { "s" },
                f.frequency,
            );
            let name = f.name.clone();
            let pcount = f.param_count;
            button(text(label).size(12))
                .on_press(EventScrEditorMessage::InsertPickedFunction(name, pcount))
                .width(Length::Fill)
                .padding([4, 8])
                .into()
        })
        .collect();

    column![
        text("Pick a function to insert:").size(13),
        text_input("Filter functions...", filter)
            .on_input(EventScrEditorMessage::PickerFilterChanged)
            .padding(6)
            .size(13),
        scrollable(column(list).spacing(2)).height(Length::Fixed(200.0)),
    ]
    .spacing(6)
    .padding(8)
    .into()
}

fn separator() -> Element<'static, EventScrEditorMessage> {
    container(text(""))
        .height(Length::Fixed(1.0))
        .width(Length::Fixed(1.0))
        .style(|_theme| container::Style {
            background: Some(Color::from_rgb(0.5, 0.5, 0.5).into()),
            ..Default::default()
        })
        .into()
}

fn empty_editor<'a>() -> Element<'a, EventScrEditorMessage> {
    container(text("No EventScript loaded. Use 'Browse' to open a file.").size(16))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
