use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::components::utils::horizontal_rule as hr;
use crate::editors::event_scr::message::EventScrEditorMessage;
use crate::editors::event_scr::state::{EventScriptEditorState, SectionTab};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{font, Alignment, Border, Color, Element, Font, Length};

pub fn view(app: &App) -> Element<'_, EventScrEditorMessage> {
    let state = &app.state.event_scr_editor;

    match &state.script_loading {
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
                SectionTab::Act => view_act_section(script),
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

fn view_act_section(
    script: &dispel_core::references::event_scr::EventScript,
) -> Element<'static, EventScrEditorMessage> {
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

    column![
        text("Action Functions").size(16).font(Font {
            weight: font::Weight::Bold,
            ..Font::DEFAULT
        }),
        header,
        scrollable(column(rows).spacing(5)).height(Length::Fill),
        row![
            button("+ Add Action").on_press(EventScrEditorMessage::ActionAdded),
            button("+ Add Raw Text").on_press(EventScrEditorMessage::ActionRawAdded),
        ]
        .spacing(10),
    ]
    .spacing(10)
    .into()
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
