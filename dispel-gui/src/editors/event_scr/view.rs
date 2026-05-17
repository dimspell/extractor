use std::sync::atomic::Ordering;

use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::components::utils::horizontal_rule as hr;
use crate::editors::event_scr::act_tree::{build_act_tree, ScriptNode};
use crate::editors::event_scr::functions::{EventScriptFunctionIndex, IndexedFunction};
use crate::editors::event_scr::message::EventScrEditorMessage;
use crate::editors::event_scr::state::{EventScriptEditorState, FunctionIndexState, SectionTab};
use crate::style;
use iced::widget::{
    button, column, container, pick_list, progress_bar, row, rule, scrollable, text, text_input,
    Space,
};
use iced::{Alignment, Color, Element, Font, Length};

pub fn view(app: &App) -> Element<'_, EventScrEditorMessage> {
    let state = &app.state.event_scr_editor;

    let base = match &state.script_loading {
        LoadingState::Loaded(script) => {
            let script_id = script.id;
            let modified_indicator = if state.modified { " ●" } else { "" };

            let save_button = button("Save")
                .on_press(EventScrEditorMessage::SaveScript)
                .style(if state.modified {
                    style::commit_button
                } else {
                    style::browse_button
                });

            let save_error: Element<EventScrEditorMessage> = if let Some(ref err) = state.save_error
            {
                text(err).size(14).style(style::primary_text).into()
            } else {
                text("").into()
            };

            let index_info: Element<EventScrEditorMessage> = match &state.index_state {
                FunctionIndexState::Loaded(index) => {
                    let count = index.functions.len();
                    row![
                        text(format!("{} function(s) indexed", count))
                            .size(12)
                            .style(style::subtle_text),
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

            let act_toolbar = row![
                button("+ Add Action")
                    .on_press(EventScrEditorMessage::ActionAdded)
                    .style(style::browse_button),
                button("+ Raw Text")
                    .on_press(EventScrEditorMessage::ActionRawAdded)
                    .style(style::browse_button),
                button("IF")
                    .on_press(EventScrEditorMessage::InsertIfBlock)
                    .style(style::chip),
                button("ELSE")
                    .on_press(EventScrEditorMessage::InsertElseBlock)
                    .style(style::chip),
                button("RET")
                    .on_press(EventScrEditorMessage::InsertReturnBlock)
                    .style(style::chip),
                button("Pick")
                    .on_press(EventScrEditorMessage::ToggleFunctionPicker)
                    .style(style::chip),
            ]
            .spacing(10);

            let tree_nodes = build_act_tree(&script.actions);
            let tree_elements = render_act_tree(&tree_nodes, &script.actions, script, state);

            let mut page_content: Vec<Element<EventScrEditorMessage>> = Vec::new();

            // ACT section — always visible, permanently expanded
            page_content.push(
                container(
                    column![
                        row![
                            text("Action Functions")
                                .size(16)
                                .style(style::section_header),
                            Space::new().width(Length::Fill),
                            act_toolbar,
                        ]
                        .align_y(Alignment::Center),
                        index_info,
                    ]
                    .spacing(6),
                )
                .style(style::panel_container)
                .padding([6, 10])
                .width(Length::Fill)
                .into(),
            );
            page_content.push(column(tree_elements).spacing(1).into());

            // Collapsible panels (all non-ACT sections)
            // Header
            page_content.push(collapsible_panel(
                SectionTab::Header,
                "Header",
                script.header_comments.len(),
                state,
                None,
                if state.panels_expanded.contains(&SectionTab::Header) {
                    Some(body_header(script))
                } else {
                    None
                },
            ));

            // Variables
            page_content.push(collapsible_panel(
                SectionTab::Var,
                "Variables",
                script.variables.len(),
                state,
                Some(EventScrEditorMessage::VariableAdded),
                if state.panels_expanded.contains(&SectionTab::Var) {
                    Some(body_var(script))
                } else {
                    None
                },
            ));

            // Map
            page_content.push(collapsible_panel(
                SectionTab::Map,
                "Map",
                script.map_content.len(),
                state,
                Some(EventScrEditorMessage::LineAdded(SectionTab::Map)),
                if state.panels_expanded.contains(&SectionTab::Map) {
                    Some(body_line(script, SectionTab::Map))
                } else {
                    None
                },
            ));

            // Chr
            page_content.push(collapsible_panel(
                SectionTab::Chr,
                "Chr",
                script.chr_content.len(),
                state,
                Some(EventScrEditorMessage::LineAdded(SectionTab::Chr)),
                if state.panels_expanded.contains(&SectionTab::Chr) {
                    Some(body_line(script, SectionTab::Chr))
                } else {
                    None
                },
            ));

            // Npc
            page_content.push(collapsible_panel(
                SectionTab::Npc,
                "Npc",
                script.npc_content.len(),
                state,
                Some(EventScrEditorMessage::LineAdded(SectionTab::Npc)),
                if state.panels_expanded.contains(&SectionTab::Npc) {
                    Some(body_line(script, SectionTab::Npc))
                } else {
                    None
                },
            ));

            // Sprites
            page_content.push(collapsible_panel(
                SectionTab::Spr,
                "Sprites",
                script.spr_content.len(),
                state,
                Some(EventScrEditorMessage::SpriteAdded),
                if state.panels_expanded.contains(&SectionTab::Spr) {
                    Some(body_spr(script))
                } else {
                    None
                },
            ));

            // Wav
            page_content.push(collapsible_panel(
                SectionTab::Wav,
                "Wav",
                script.wav_content.len(),
                state,
                Some(EventScrEditorMessage::LineAdded(SectionTab::Wav)),
                if state.panels_expanded.contains(&SectionTab::Wav) {
                    Some(body_line(script, SectionTab::Wav))
                } else {
                    None
                },
            ));

            column![
                row![
                    text(format!("EventScript [{}]", script_id)).size(20),
                    text(modified_indicator)
                        .size(20)
                        .style(style::section_header),
                    Space::new().width(Length::Fill),
                    save_button,
                ]
                .align_y(Alignment::Center),
                scrollable(column(page_content).spacing(10)).height(Length::Fill),
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

    let base: Element<'_, EventScrEditorMessage> = if state.picker_open {
        let picker_content: Element<'_, EventScrEditorMessage> =
            container(view_function_picker(state))
                .style(style::modal_container)
                .max_width(520)
                .into();
        crate::components::modal::modal(
            base,
            picker_content,
            || EventScrEditorMessage::ToggleFunctionPicker,
            0.3,
        )
    } else {
        base
    };

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

fn collapsible_panel<'a>(
    tab: SectionTab,
    label: &'a str,
    count: usize,
    state: &EventScriptEditorState,
    add_msg: Option<EventScrEditorMessage>,
    body: Option<Vec<Element<'a, EventScrEditorMessage>>>,
) -> Element<'a, EventScrEditorMessage> {
    let expanded = state.panels_expanded.contains(&tab);
    let arrow = if expanded { "▼" } else { "▶" };
    let count_str = format!(" ({})", count);

    let mut header_children: Vec<Element<EventScrEditorMessage>> = vec![
        text(arrow).size(13).into(),
        text(label).size(14).style(style::section_header).into(),
        text(count_str).size(12).style(style::subtle_text).into(),
        Space::new().width(Length::Fill).into(),
    ];

    if let Some(msg) = add_msg {
        header_children.push(
            button(text("+").size(14))
                .on_press(msg)
                .style(style::chip)
                .padding([1, 8])
                .into(),
        );
    }

    let header = button(row(header_children).spacing(8).align_y(Alignment::Center))
        .on_press(EventScrEditorMessage::TogglePanel(tab))
        .style(style::tab_button)
        .padding([4, 8])
        .width(Length::Fill);

    if let Some(body_content) = body {
        container(
            column![
                header,
                container(column(body_content).spacing(4))
                    .padding([4, 12])
                    .width(Length::Fill),
            ]
            .spacing(0),
        )
        .style(style::panel_container)
        .width(Length::Fill)
        .into()
    } else {
        container(header)
            .style(style::panel_container)
            .width(Length::Fill)
            .into()
    }
}

fn body_header(
    script: &dispel_core::references::event_scr::EventScript,
) -> Vec<Element<'static, EventScrEditorMessage>> {
    script
        .header_comments
        .iter()
        .map(|line| text(line.clone()).into())
        .collect()
}

fn body_var(
    script: &dispel_core::references::event_scr::EventScript,
) -> Vec<Element<'static, EventScrEditorMessage>> {
    let header = row![
        text("Name")
            .style(style::section_header)
            .width(Length::FillPortion(2)),
        text("Value")
            .style(style::section_header)
            .width(Length::FillPortion(2)),
        text("Actions")
            .style(style::section_header)
            .width(Length::FillPortion(1)),
    ]
    .spacing(10);

    let mut rows: Vec<Element<EventScrEditorMessage>> = vec![header.into()];

    for (i, var) in script.variables.iter().enumerate() {
        rows.push(
            row![
                text_input("", &var.name)
                    .on_input(move |s| EventScrEditorMessage::VariableNameChanged(i, s))
                    .width(Length::FillPortion(2)),
                text_input("", &var.value)
                    .on_input(move |s| EventScrEditorMessage::VariableValueChanged(i, s))
                    .width(Length::FillPortion(2)),
                button("Delete")
                    .on_press(EventScrEditorMessage::VariableDeleted(i))
                    .width(Length::FillPortion(1))
                    .style(style::normal_row_button),
            ]
            .spacing(10)
            .into(),
        );
    }

    rows
}

fn body_line(
    script: &dispel_core::references::event_scr::EventScript,
    section: SectionTab,
) -> Vec<Element<'static, EventScrEditorMessage>> {
    let lines: &Vec<String> = match section {
        SectionTab::Map => &script.map_content,
        SectionTab::Chr => &script.chr_content,
        SectionTab::Npc => &script.npc_content,
        SectionTab::Wav => &script.wav_content,
        _ => return vec![text("Invalid section").into()],
    };

    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            row![
                text_input("", line)
                    .on_input(move |s| EventScrEditorMessage::LineContentChanged(section, i, s))
                    .width(Length::FillPortion(5)),
                button("Delete")
                    .on_press(EventScrEditorMessage::LineDeleted(section, i))
                    .width(Length::FillPortion(1))
                    .style(style::normal_row_button),
            ]
            .spacing(10)
            .into()
        })
        .collect()
}

fn body_spr(
    script: &dispel_core::references::event_scr::EventScript,
) -> Vec<Element<'static, EventScrEditorMessage>> {
    let header = row![
        text("Alias")
            .style(style::section_header)
            .width(Length::FillPortion(2)),
        text("Filename")
            .style(style::section_header)
            .width(Length::FillPortion(2)),
        text("Actions")
            .style(style::section_header)
            .width(Length::FillPortion(1)),
    ]
    .spacing(10);

    let mut rows: Vec<Element<EventScrEditorMessage>> = vec![header.into()];

    for (i, spr) in script.spr_content.iter().enumerate() {
        rows.push(
            row![
                text_input("", &spr.sprite_alias)
                    .on_input(move |s| EventScrEditorMessage::SpriteAliasChanged(i, s))
                    .width(Length::FillPortion(2)),
                text_input("", &spr.sprite_file)
                    .on_input(move |s| EventScrEditorMessage::SpriteFileChanged(i, s))
                    .width(Length::FillPortion(2)),
                button("Delete")
                    .on_press(EventScrEditorMessage::SpriteDeleted(i))
                    .width(Length::FillPortion(1))
                    .style(style::normal_row_button),
            ]
            .spacing(10)
            .into(),
        );
    }

    rows
}

// ── Row building helpers ────────────────────────────────────────────────────

fn indent_guides<'a>(depth: usize) -> Element<'a, EventScrEditorMessage> {
    if depth == 0 {
        return Space::new().width(Length::Fixed(0.0)).into();
    }
    row((0..depth)
        .map(|_| rule::vertical(1).into())
        .collect::<Vec<Element<EventScrEditorMessage>>>())
    .spacing(0)
    .width(Length::Shrink)
    .into()
}

fn move_buttons(index: usize) -> Element<'static, EventScrEditorMessage> {
    row![
        button(text("↑").size(12))
            .on_press(EventScrEditorMessage::MoveActionUp(index))
            .style(style::move_button)
            .padding([1, 3]),
        button(text("↓").size(12))
            .on_press(EventScrEditorMessage::MoveActionDown(index))
            .style(style::move_button)
            .padding([1, 3]),
    ]
    .spacing(0)
    .into()
}

fn del_button(index: usize) -> Element<'static, EventScrEditorMessage> {
    button("Del")
        .on_press(EventScrEditorMessage::ActionDeleted(index))
        .style(style::normal_row_button)
        .into()
}

fn render_error_aware<'a>(
    _index: usize,
    is_error: bool,
    row: impl Into<Element<'a, EventScrEditorMessage>>,
) -> Element<'a, EventScrEditorMessage> {
    if is_error {
        container(row)
            .style(style::error_row_border)
            .padding([2, 6])
            .width(Length::Fill)
            .into()
    } else {
        container(row).padding([2, 6]).width(Length::Fill).into()
    }
}

fn build_prefix_options(
    script: &dispel_core::references::event_scr::EventScript,
    act: &dispel_core::references::event_scr::ActionFunction,
) -> Vec<String> {
    let mut aliases: Vec<String> = script
        .spr_content
        .iter()
        .map(|s| s.sprite_alias.clone())
        .collect();
    aliases.sort();
    aliases.dedup();

    let current = act.prefix.clone().unwrap_or_else(|| "(none)".to_string());
    if current != "(none)" && !aliases.contains(&current) {
        aliases.insert(0, current);
    }
    aliases.push("(none)".to_string());
    aliases
}

fn render_inline_suggestions<'a>(
    action_index: usize,
    filter: &str,
    index_data: &'a EventScriptFunctionIndex,
) -> Element<'a, EventScrEditorMessage> {
    let filtered: Vec<&IndexedFunction> = index_data
        .functions
        .iter()
        .filter(|f| filter.is_empty() || f.name.to_lowercase().contains(&filter.to_lowercase()))
        .take(8)
        .collect();

    if filtered.is_empty() {
        return container(text("No matches").size(11).style(style::subtle_text))
            .padding([2, 12])
            .into();
    }

    let items: Vec<Element<EventScrEditorMessage>> = filtered
        .into_iter()
        .map(|f| {
            let label = format!(
                "{} ({} param{})",
                f.name,
                f.param_count,
                if f.param_count == 1 { "" } else { "s" }
            );
            let name = f.name.clone();
            button(text(label).size(11).font(Font::MONOSPACE))
                .on_press(EventScrEditorMessage::SuggestionSelect(action_index, name))
                .width(Length::Fill)
                .padding([2, 8])
                .style(style::chip)
                .into()
        })
        .collect();

    container(column(items).spacing(1))
        .style(style::modal_container)
        .padding([4, 8])
        .max_width(400)
        .into()
}

// ── Tree rendering ──────────────────────────────────────────────────────────

fn render_act_tree<'a>(
    nodes: &[ScriptNode],
    actions: &'a [dispel_core::references::event_scr::ActionFunction],
    script: &'a dispel_core::references::event_scr::EventScript,
    state: &'a EventScriptEditorState,
) -> Vec<Element<'a, EventScrEditorMessage>> {
    let mut elements = Vec::new();
    for node in nodes {
        match node {
            ScriptNode::Statement {
                action_index,
                depth,
            } => {
                elements.push(render_action_row(
                    *action_index,
                    *depth,
                    actions,
                    script,
                    state,
                ));
            }
            ScriptNode::Block {
                open_index,
                close_index,
                depth,
                children,
            } => {
                let folded = state.act_folded.contains(open_index);
                elements.push(render_open_row(*open_index, *close_index, *depth, folded));
                if folded {
                    let hidden = count_hidden(children);
                    elements.push(render_folded_hint(*depth + 1, hidden));
                } else {
                    elements.extend(render_act_tree(children, actions, script, state));
                    if *close_index != usize::MAX {
                        elements.push(render_close_row(*depth, *close_index));
                    }
                }
            }
        }
    }
    elements
}

fn render_action_row<'a>(
    index: usize,
    depth: usize,
    actions: &'a [dispel_core::references::event_scr::ActionFunction],
    script: &'a dispel_core::references::event_scr::EventScript,
    state: &'a EventScriptEditorState,
) -> Element<'a, EventScrEditorMessage> {
    let act = &actions[index];

    let is_error = state.act_parse_errors.iter().any(|(idx, _)| *idx == index);

    let content: Element<EventScrEditorMessage> = if let Some(ref raw) = act.raw_content {
        if let Some(cond) = raw.strip_prefix("if(").and_then(|s| s.strip_suffix(')')) {
            render_error_aware(
                index,
                is_error,
                row![
                    indent_guides(depth),
                    badge("IF", "if"),
                    text("Cond:").size(12).style(style::subtle_text),
                    text_input("condition", cond)
                        .on_input(move |s| EventScrEditorMessage::IfConditionChanged(index, s))
                        .width(Length::Fill)
                        .font(Font::MONOSPACE),
                    move_buttons(index),
                    del_button(index),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
        } else if raw == "else" {
            render_error_aware(
                index,
                is_error,
                row![
                    indent_guides(depth),
                    badge("ELSE", "else"),
                    Space::new().width(Length::Fill),
                    move_buttons(index),
                    del_button(index),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
        } else if let Some(val) = raw
            .strip_prefix("return(")
            .and_then(|s| s.strip_suffix(')'))
        {
            render_error_aware(
                index,
                is_error,
                row![
                    indent_guides(depth),
                    badge("RET", "ret"),
                    text("Val:").size(12).style(style::subtle_text),
                    text_input("value", val)
                        .on_input(move |s| EventScrEditorMessage::ReturnValueChanged(index, s))
                        .width(Length::Fill)
                        .font(Font::MONOSPACE),
                    move_buttons(index),
                    del_button(index),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
        } else {
            render_error_aware(
                index,
                is_error,
                row![
                    indent_guides(depth),
                    badge("TEXT", "text"),
                    text_input("", raw)
                        .on_input(move |s| {
                            EventScrEditorMessage::ActionRawContentChanged(index, s)
                        })
                        .width(Length::Fill)
                        .font(Font::MONOSPACE),
                    move_buttons(index),
                    del_button(index),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
        }
    } else {
        let params_str = act.parameters.join(", ");
        let prefix_options = build_prefix_options(script, act);
        let current_prefix = act.prefix.clone().unwrap_or_else(|| "(none)".to_string());

        let suggestion_row = if state.suggestion_visible
            && state.suggestion_active_index == Some(index)
            && !act.function_name.is_empty()
        {
            if let FunctionIndexState::Loaded(ref index_data) = state.index_state {
                let picker = render_inline_suggestions(index, &act.function_name, index_data);
                Some(picker)
            } else {
                None
            }
        } else {
            None
        };

        let mut func_row = column![render_error_aware(
            index,
            is_error,
            row![
                indent_guides(depth),
                badge("FUNC", "func"),
                pick_list(prefix_options, Some(current_prefix.clone()), move |v| {
                    let opt = if v == "(none)" { None } else { Some(v) };
                    EventScrEditorMessage::ActionPrefixPicked(index, opt)
                },)
                .text_size(12)
                .padding([2, 6]),
                text("~").size(13).style(style::subtle_text),
                text_input("function", &act.function_name)
                    .on_input(move |s| { EventScrEditorMessage::ActionFunctionChanged(index, s) })
                    .width(Length::FillPortion(3))
                    .font(Font::MONOSPACE),
                text("(").size(13).style(style::subtle_text),
                text_input("params", &params_str)
                    .on_input(move |s| { EventScrEditorMessage::ActionParamsChanged(index, s) })
                    .width(Length::FillPortion(2))
                    .font(Font::MONOSPACE),
                text(")").size(13).style(style::subtle_text),
                move_buttons(index),
                del_button(index),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        ),];

        if let Some(sugg) = suggestion_row {
            func_row = func_row.push(sugg);
        }

        return container(func_row)
            .padding([3, 8])
            .width(Length::Fill)
            .into();
    };

    container(content)
        .padding([3, 8])
        .width(Length::Fill)
        .into()
}

fn render_open_row<'a>(
    index: usize,
    close_index: usize,
    depth: usize,
    folded: bool,
) -> Element<'a, EventScrEditorMessage> {
    let arrow = if folded { "▶" } else { "▼" };
    container(
        row![
            indent_guides(depth),
            button(text(arrow).size(11))
                .on_press(EventScrEditorMessage::ToggleFold(index))
                .style(style::fold_button)
                .padding([1, 3]),
            text("{").size(13).style(style::subtle_text),
            Space::new().width(Length::Fill),
            button(text("+").size(13))
                .on_press(EventScrEditorMessage::InsertWithPickerAt(close_index))
                .style(style::chip)
                .padding([1, 7]),
            button(text("↑").size(12))
                .on_press(EventScrEditorMessage::MoveActionUp(index))
                .style(style::move_button)
                .padding([1, 3]),
            button(text("↓").size(12))
                .on_press(EventScrEditorMessage::MoveActionDown(index))
                .style(style::move_button)
                .padding([1, 3]),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    )
    .padding([3, 8])
    .width(Length::Fill)
    .into()
}

fn render_close_row<'a>(depth: usize, index: usize) -> Element<'a, EventScrEditorMessage> {
    container(
        row![
            indent_guides(depth),
            text("}").size(13).style(style::subtle_text),
            Space::new().width(Length::Fill),
            button(text("↑").size(12))
                .on_press(EventScrEditorMessage::MoveActionUp(index))
                .style(style::move_button)
                .padding([1, 3]),
            button(text("↓").size(12))
                .on_press(EventScrEditorMessage::MoveActionDown(index))
                .style(style::move_button)
                .padding([1, 3]),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([3, 8])
    .width(Length::Fill)
    .into()
}

fn render_folded_hint<'a>(depth: usize, count: usize) -> Element<'a, EventScrEditorMessage> {
    let label = if count == 1 {
        "… 1 action …"
    } else {
        "… actions …"
    };
    container(
        row![
            indent_guides(depth),
            text(label).size(11).style(style::subtle_text),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([2, 8])
    .width(Length::Fill)
    .into()
}

fn count_hidden(nodes: &[ScriptNode]) -> usize {
    let mut count = 0;
    for node in nodes {
        match node {
            ScriptNode::Statement { .. } => count += 1,
            ScriptNode::Block { children, .. } => {
                count += count_hidden(children);
                count += 1;
            }
        }
    }
    count
}

fn badge<'a>(label: &'static str, kind: &str) -> Element<'a, EventScrEditorMessage> {
    let s = match kind {
        "if" => style::badge_if_container,
        "else" => style::badge_else_container,
        "ret" => style::badge_ret_container,
        "func" => style::badge_func_container,
        "text" => style::badge_text_container,
        _ => style::badge_container,
    };
    container(text(label).size(11))
        .padding([1, 5])
        .style(s)
        .into()
}

fn view_status_bar(state: &EventScriptEditorState) -> Element<'static, EventScrEditorMessage> {
    let file_path = state
        .file_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "No file loaded".to_string());

    let encoding_badge = button(text("EUC-KR").size(12))
        .padding([2, 6])
        .style(style::chip);

    let modified_text = if state.modified {
        text("Modified").size(12).style(style::section_header)
    } else {
        text("Saved").size(12).style(style::subtle_text)
    };

    let error_count = state.act_parse_errors.len();
    let error_text = if error_count > 0 {
        text(format!("{} error(s)", error_count))
            .size(12)
            .style(style::primary_text)
    } else {
        text("No errors").size(12).style(style::subtle_text)
    };

    let shortcut_text = text("Ctrl+S to save").size(11).style(style::subtle_text);

    container(
        row![
            text(file_path)
                .size(12)
                .style(style::subtle_text)
                .width(Length::Fill),
            encoding_badge,
            modified_text,
            error_text,
            shortcut_text,
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .style(style::status_bar)
    .padding([4, 12])
    .width(Length::Fill)
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
        progress_bar(0.0..=1.0, fraction).style(style::primary_progress_bar),
        text(format!("{}% — {} / {} files", pct, processed, total)).size(13),
        text(current_file).size(11).style(style::subtle_text),
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

fn empty_editor<'a>() -> Element<'a, EventScrEditorMessage> {
    container(text("No EventScript loaded. Use 'Browse' to open a file.").size(16))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
