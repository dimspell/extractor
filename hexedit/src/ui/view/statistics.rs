//! Byte statistics & entropy panel view.
//!
//! Displays a byte-frequency histogram (simplified bar chart), Shannon entropy,
//! min/max/mean/median, structure heuristics, and string-detection results.

use iced::widget::space::Space;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{color, Element, Fill, Font, Length};

use crate::domain::byte_stats::{
    ByteStatistics, RowEntropyCache, StructureHeuristic,
};
use crate::state::HexEditorState;
use crate::{HexEditorMessage, HexProvider};

/// Width of the histogram bar area in "█" character widths.
const HIST_BAR_WIDTH: usize = 30;

pub fn view(editor: &HexEditorState) -> Element<'_, HexEditorMessage> {
    let header = header_row(editor);
    let header = container(header).padding([4, 12]).width(Fill);

    let body: Element<'_, HexEditorMessage> = if editor.provider.is_empty() {
        text("(empty file)").size(11).font(Font::MONOSPACE).into()
    } else {
        let mut col = column![].spacing(4).padding([4, 12]);

        // ── File-level stats ─────────────────────────────────────────────
        match &editor.file_stats {
            Some(stats) => {
                col = col.push(stats_section(stats));
            }
            None => {
                col = col.push(hint_row("Click \"Analyze File\" to compute statistics"));
            }
        }

        // ── Selection-level stats ────────────────────────────────────────
        if !editor.selection.is_single() {
            match &editor.selection_stats {
                Some(stats) => {
                    col = col.push(selection_section(stats));
                }
                None => {
                    col = col
                        .push(hint_row("Click \"Analyze Selection\" for selection-only stats"));
                }
            }
        }

        // ── Row entropies overview ──────────────────────────────────────
        if let Some(ref re) = editor.row_entropies {
            col = col.push(row_entropy_summary(re));
        }

        col.into()
    };

    let content: Element<'_, HexEditorMessage> = scrollable(column![header, body])
        .height(Length::Fill)
        .into();

    container(content).width(Fill).height(Fill).into()
}

fn header_row(editor: &HexEditorState) -> Element<'_, HexEditorMessage> {
    let analyze_file_btn: Element<'_, HexEditorMessage> = if !editor.provider.is_empty() {
        button(text("Analyze File").size(9).font(Font::MONOSPACE))
            .padding([2, 8])
            .on_press(HexEditorMessage::AnalyzeFile)
            .into()
    } else {
        Space::default().width(0).into()
    };

    let analyze_sel_btn: Element<'_, HexEditorMessage> =
        if !editor.selection.is_single() && !editor.provider.is_empty() {
            button(text("Analyze Selection").size(9).font(Font::MONOSPACE))
                .padding([2, 8])
                .on_press(HexEditorMessage::AnalyzeSelection)
                .into()
        } else {
            Space::default().width(0).into()
        };

    row![
        text("Byte Statistics").size(11).font(Font::MONOSPACE),
        Space::default().width(Fill),
        analyze_file_btn,
        analyze_sel_btn,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Render the file-level statistics section.
fn stats_section(stats: &ByteStatistics) -> Element<'_, HexEditorMessage> {
    section_inner("File", stats)
}

/// Render the selection-level statistics section.
fn selection_section(stats: &ByteStatistics) -> Element<'_, HexEditorMessage> {
    section_inner("Selection", stats)
}

/// Shared renderer for a statistics group.
fn section_inner<'a>(label: &'a str, stats: &'a ByteStatistics) -> Element<'a, HexEditorMessage> {
    let mut col = column![].spacing(2);

    // Section label.
    col = col.push(
        text(format!("── {label} ──"))
            .size(9)
            .font(Font::MONOSPACE)
            .color(color!(0x8a7a6a)),
    );

    // Top-level metrics in a compact grid.
    let total_str = format!("{} B", stats.total);
    let entropy_str = format!("{:.4}", stats.entropy);
    let min_str = format!("0x{:02X}", stats.min);
    let max_str = format!("0x{:02X}", stats.max);

    col = col.push(
        row![
            metric("Total", total_str),
            metric("Entropy", entropy_str),
            metric("Min", min_str),
            metric("Max", max_str),
        ]
        .spacing(12),
    );

    let mean_str = format!("{:.2}", stats.mean);
    let median_str = format!("0x{:02X}", stats.median);
    let nulls_str = format!(
        "{} ({:.1}%)",
        stats.null_count,
        stats.null_count as f64 / stats.total.max(1) as f64 * 100.0
    );
    let printable_str = format!(
        "{} ({:.1}%)",
        stats.printable_count,
        stats.printable_count as f64 / stats.total.max(1) as f64 * 100.0
    );

    col = col.push(
        row![
            metric("Mean", mean_str),
            metric("Median", median_str),
            metric("Nulls", nulls_str),
            metric("Printable", printable_str),
        ]
        .spacing(12),
    );

    // Structure heuristic.
    col = col.push(structure_row(&stats.structure));

    // High-ASCII count.
    let high_ascii_str = format!(
        "{} ({:.1}%)",
        stats.high_ascii_count,
        stats.high_ascii_count as f64 / stats.total.max(1) as f64 * 100.0
    );
    col = col.push(metric_row("High ASCII (0x80+)", high_ascii_str));

    // Entropy classification badge.
    let classification = if stats.entropy > 7.2 {
        "[HIGH] Likely compressed/encrypted"
    } else if stats.entropy < 2.4 {
        "[LOW]  Likely sparse/padding"
    } else if stats.entropy > 5.5 {
        "[MIX]  Mixed binary data"
    } else {
        "[OK]   Structured / text data"
    };
    col = col.push(
        text(classification)
            .size(10)
            .font(Font::MONOSPACE),
    );

    // Byte frequency histogram.
    col = col.push(histogram_view(stats));

    col.into()
}

/// Render a simplified byte histogram showing the distribution across 16 groups
/// (high nybbles).
fn histogram_view(stats: &ByteStatistics) -> Element<'_, HexEditorMessage> {
    if stats.total == 0 {
        return text("").into();
    }

    // Aggregate by high nybble.
    let mut groups = [0u64; 16];
    for (byte, &count) in stats.histogram.iter().enumerate() {
        groups[byte >> 4] += count;
    }

    let max_group = *groups.iter().max().unwrap_or(&1).max(&1);

    let mut col = column![].spacing(1);
    col = col.push(
        text("Byte distribution (high nybble)")
            .size(9)
            .font(Font::MONOSPACE)
            .color(color!(0x8a7a6a)),
    );

    for (nybble, &count) in groups.iter().enumerate() {
        let label = format!("{nybble:X}x__");
        let fraction = count as f64 / max_group as f64;
        let bar_len = (fraction * HIST_BAR_WIDTH as f64).round() as usize;
        let bar: String = "█".repeat(bar_len);
        let pct = format!(" {:.1}%", count as f64 / stats.total as f64 * 100.0);

        col = col.push(
            row![
                text(label).size(9).font(Font::MONOSPACE).width(36),
                text(bar).size(9).font(Font::MONOSPACE).color(bar_color(nybble)),
                text(pct).size(9).font(Font::MONOSPACE).color(color!(0x7a6f64)),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        );
    }

    col.into()
}

/// Colour for a histogram bar based on nybble range.
fn bar_color(nybble: usize) -> iced::Color {
    match nybble {
        0 => color!(0x6a5a4a),    // sparse/padding
        1..=3 => color!(0x7a8a5a), // low values
        4..=7 => color!(0x5a8a7a), // mid-low
        8..=11 => color!(0x8a7a4a), // mid-high
        12..=15 => color!(0xaa5a4a), // high values
        _ => color!(0x8a7a6a),
    }
}

/// Render the entropy classification as a colour-coded row.
fn structure_row(structure: &StructureHeuristic) -> Element<'_, HexEditorMessage> {
    let (label, color_val) = match structure {
        StructureHeuristic::Uniform(val) => (
            format!("Structure: Uniform (all bytes = 0x{val:02X})"),
            color!(0x6a8a5a),
        ),
        StructureHeuristic::HighEntropy => (
            "Structure: High entropy — likely compressed/encrypted".to_string(),
            color!(0xaa4a3a),
        ),
        StructureHeuristic::LowEntropy => (
            "Structure: Low entropy — likely sparse/padding".to_string(),
            color!(0x5a7a8a),
        ),
        StructureHeuristic::Mixed => (
            "Structure: Mixed — typical structured file".to_string(),
            color!(0x8a7a6a),
        ),
    };

    text(label)
        .size(10)
        .font(Font::MONOSPACE)
        .color(color_val)
        .into()
}

/// Summary of per-row entropy values.
fn row_entropy_summary(cache: &RowEntropyCache) -> Element<'_, HexEditorMessage> {
    let avg: f64 = if cache.rows.is_empty() {
        0.0
    } else {
        cache.rows.iter().map(|(_, e)| e).sum::<f64>() / cache.rows.len() as f64
    };

    let rows_str = format!("{}", cache.rows.len());
    let min_str = format!("{:.4}", cache.min_entropy);
    let max_str = format!("{:.4}", cache.max_entropy);
    let avg_str = format!("{:.4}", avg);

    column![
        text("── Row Entropy ──")
            .size(9)
            .font(Font::MONOSPACE)
            .color(color!(0x8a7a6a)),
        row![
            metric("Rows", rows_str),
            metric("Min row", min_str),
            metric("Max row", max_str),
            metric("Avg row", avg_str),
        ]
        .spacing(12),
    ]
    .spacing(2)
    .into()
}

// ── Small helpers ──────────────────────────────────────────────────────────

fn metric(label: &str, value: String) -> Element<'_, HexEditorMessage> {
    column![
        text(label).size(8).font(Font::MONOSPACE).color(color!(0x7a6f64)),
        text(value).size(10).font(Font::MONOSPACE),
    ]
    .spacing(0)
    .into()
}

fn metric_row(label: &str, value: String) -> Element<'_, HexEditorMessage> {
    row![
        text(label)
            .size(10)
            .font(Font::MONOSPACE)
            .color(color!(0x7a6f64))
            .width(160),
        text(value).size(10).font(Font::MONOSPACE),
    ]
    .spacing(8)
    .into()
}

fn hint_row(msg: &str) -> Element<'_, HexEditorMessage> {
    text(msg)
        .size(10)
        .font(Font::MONOSPACE)
        .color(color!(0x7a6f64))
        .into()
}
