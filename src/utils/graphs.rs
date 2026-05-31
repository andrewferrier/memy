use core::fmt::Write as _;
use terminal_size::{Width, terminal_size};

const BAR_CHAR: &str = "█";
const MAX_CHART_HEIGHT: usize = 10;

pub const COL_WIDTH: usize = 9;

pub fn get_terminal_width() -> usize {
    const DEFAULT_TERMINAL_WIDTH: usize = 80;
    terminal_size().map_or(DEFAULT_TERMINAL_WIDTH, |(Width(w), _)| usize::from(w))
}

/// Round `max_val` up to a "nice" Y-axis step so labels are clean multiples
/// (e.g. 10, 20, 50, 100, 200 …).  `max_rows` caps the number of rows.
const fn nice_step(max_val: usize, max_rows: usize) -> usize {
    if max_val == 0 || max_rows == 0 {
        return 1;
    }
    let rough = max_val.div_ceil(max_rows);
    if rough <= 1 {
        return 1;
    }
    // Find the largest power of 10 that is <= rough.
    let mut magnitude = 1_usize;
    while magnitude * 10 <= rough {
        magnitude *= 10;
    }
    // Normalised fractional step ∈ [1, 9].
    let normalized = rough.div_ceil(magnitude);
    let nice_normalized = if normalized <= 1 {
        1
    } else if normalized <= 2 {
        2
    } else if normalized <= 5 {
        5
    } else {
        10
    };
    nice_normalized * magnitude
}

pub fn render_bar_chart(title: &str, entries: &[(String, usize)], terminal_width: usize) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let max_count = entries.iter().map(|(_, c)| *c).max().unwrap_or(1);
    let max_label_len = entries.iter().map(|(l, _)| l.len()).max().unwrap_or(1);
    let count_digits = max_count.to_string().len();

    // Layout: "LABEL │ BAR COUNT\n"
    // Overhead: max_label_len + 1 (space) + 1 (│) + 1 (space) + 1 (space) + count_digits
    let overhead = max_label_len + 4 + 1 + count_digits;
    let bar_width = terminal_width.saturating_sub(overhead).max(5);

    let mut output = format!("{title}:\n");
    for (label, count) in entries {
        let bar_len = (count * bar_width).checked_div(max_count).unwrap_or(0);
        let bar = BAR_CHAR.repeat(bar_len);
        let _ = writeln!(
            output,
            "{label:<max_label_len$} │ {bar:<bar_width$} {count}"
        );
    }

    output
}

pub fn render_column_chart(
    title: &str,
    entries: &[(String, usize)],
    terminal_width: usize,
) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let max_count = entries.iter().map(|(_, c)| *c).max().unwrap_or(1);

    let step = nice_step(max_count, MAX_CHART_HEIGHT);
    let chart_height = max_count.div_ceil(step);
    let y_top = chart_height * step; // top of Y-axis (a clean multiple)

    let y_top_digits = y_top.to_string().len();
    // y_axis_width: digits + " ┤"
    let y_axis_width = y_top_digits + 2;

    let max_cols = (terminal_width.saturating_sub(y_axis_width)) / COL_WIDTH;
    let display_entries = if entries.len() > max_cols.max(1) {
        &entries[entries.len() - max_cols.max(1)..]
    } else {
        entries
    };

    let mut output = format!("{title}:\n");

    // Count row above the bars — each count right-aligned within its column slot.
    let label_content_width = COL_WIDTH - 1;
    let _ = write!(output, "{}", " ".repeat(y_axis_width));
    for (_, count) in display_entries {
        let count_str = count.to_string();
        if count_str.len() >= label_content_width {
            let _ = write!(output, "{} ", &count_str[..label_content_width]);
        } else {
            let _ = write!(output, "{count_str:>label_content_width$} ");
        }
    }
    let _ = writeln!(output);

    for row in 0..chart_height {
        // Row 0 is the top; the Y label for each row is a multiple of step.
        let y_val = (chart_height - row) * step;
        let _ = write!(output, "{y_val:>y_top_digits$} ┤");
        for (_, count) in display_entries {
            // How many rows does this bar fill from the bottom?
            let filled_rows = count.div_ceil(step); // ceil(count / step)
            if row >= chart_height.saturating_sub(filled_rows) {
                let _ = write!(output, "{} ", BAR_CHAR.repeat(COL_WIDTH - 1));
            } else {
                let _ = write!(output, "{}", " ".repeat(COL_WIDTH));
            }
        }
        let _ = writeln!(output);
    }

    // Bottom axis
    let axis_len = display_entries.len() * COL_WIDTH;
    let _ = writeln!(
        output,
        "{} └{}",
        " ".repeat(y_top_digits),
        "─".repeat(axis_len)
    );

    // Label row — content fits in COL_WIDTH-1 chars, leaving 1 char as separator.
    let _ = write!(output, "{}", " ".repeat(y_axis_width));
    for (label, _) in display_entries {
        if label.len() >= label_content_width {
            let _ = write!(output, "{} ", &label[..label_content_width]);
        } else {
            let _ = write!(output, "{label:<label_content_width$} ");
        }
    }
    let _ = writeln!(output);

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nice_step_zero() {
        assert_eq!(nice_step(0, 10), 1);
    }

    #[test]
    fn test_nice_step_exact_fit() {
        assert_eq!(nice_step(100, 10), 10);
    }

    #[test]
    fn test_nice_step_rounds_up_to_nice_multiple() {
        assert_eq!(nice_step(101, 10), 20);
    }

    #[test]
    fn test_nice_step_small_values() {
        assert_eq!(nice_step(5, 10), 1);
    }

    #[test]
    fn test_nice_step_large_span() {
        assert_eq!(nice_step(5000, 10), 500);
    }

    #[allow(clippy::unwrap_used, reason = "unwrap OK inside tests")]
    #[test]
    fn test_render_bar_chart_basic() {
        let entries = vec![("a".to_owned(), 10), ("b".to_owned(), 5)];
        let output = render_bar_chart("Test Chart", &entries, 80);
        assert!(output.starts_with("Test Chart:\n"));
        assert!(output.contains("10"));
        assert!(output.contains('5'));
        assert!(output.contains('│'));
        // No leading whitespace before labels
        let data_line = output.lines().nth(1).unwrap_or("");
        assert!(
            !data_line.starts_with(' '),
            "Data lines should not have leading whitespace: {data_line:?}"
        );
    }

    #[test]
    fn test_render_bar_chart_no_bottom_line() {
        let entries = vec![("1".to_owned(), 3), ("2-3".to_owned(), 2)];
        let output = render_bar_chart("Count Distribution", &entries, 80);
        assert!(
            !output.contains('└'),
            "Bottom axis line should not be present: {output}"
        );
    }

    #[test]
    fn test_render_bar_chart_empty() {
        let output = render_bar_chart("Empty", &[], 80);
        assert!(output.is_empty());
    }

    #[allow(clippy::unwrap_used, reason = "unwrap OK inside tests")]
    #[test]
    fn test_render_column_chart_basic() {
        let entries = vec![("2024-01".to_owned(), 10), ("2024-02".to_owned(), 5)];
        let output = render_column_chart("Test Chart", &entries, 80);
        assert!(output.starts_with("Test Chart:\n"));
        assert!(output.contains('┤'), "Expected Y-axis separator");
        assert!(output.contains('└'), "Expected bottom axis corner");
        assert!(output.contains("2024-01"));
        assert!(output.contains("2024-02"));
        assert!(
            output.contains("10"),
            "Expected count '10' in output: {output}"
        );
        assert!(
            output.contains(" 5"),
            "Expected count '5' in output: {output}"
        );
    }

    #[test]
    fn test_render_column_chart_empty() {
        let output = render_column_chart("Empty", &[], 80);
        assert!(output.is_empty());
    }

    #[test]
    fn test_render_column_chart_single_column() {
        let entries = vec![("2024-01".to_owned(), 5)];
        let output = render_column_chart("Test", &entries, 80);
        assert!(!output.is_empty());
        assert!(output.contains('┤'));
    }

    #[test]
    fn test_render_column_chart_y_axis_multiples() {
        let entries = vec![
            ("a".to_owned(), 10),
            ("b".to_owned(), 5),
            ("c".to_owned(), 3),
        ];
        let output = render_column_chart("Test", &entries, 80);
        for line in output.lines() {
            if let Some(before) = line.find('┤') {
                let y_label = line[..before].trim();
                assert!(
                    y_label.parse::<usize>().is_ok(),
                    "Non-integer Y label: {y_label:?} in line: {line:?}"
                );
            }
        }
    }

    #[test]
    fn test_render_column_chart_label_whitespace() {
        // 8-char labels (like "2024-W04") should have a visible gap between them.
        let entries: Vec<(String, usize)> = vec![
            ("2024-W01".to_owned(), 5),
            ("2024-W02".to_owned(), 3),
            ("2024-W03".to_owned(), 7),
        ];
        let output = render_column_chart("Test", &entries, 120);
        let label_row = output.lines().last().unwrap_or("");
        assert!(
            label_row.contains("2024-W01 "),
            "Expected space after 8-char label, got: {label_row:?}"
        );
    }

    #[test]
    fn test_render_column_chart_truncates_to_fit_width() {
        let entries: Vec<(String, usize)> = (0..50_usize).map(|i| (format!("{i:08}"), 1)).collect();
        let output = render_column_chart("Test", &entries, 30);
        assert!(
            output.contains("00000049"),
            "Most recent entry should appear: {output}"
        );
        assert!(
            !output.contains("00000000"),
            "Oldest entry should be truncated away: {output}"
        );
    }
}
