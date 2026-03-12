use crate::tui::app::{AppState, FocusedPanel};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct MetricsPanelWidget<'a> {
    pub state: &'a AppState,
}

impl<'a> Widget for MetricsPanelWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let focused = self.state.focused_panel == FocusedPanel::Metrics;
        let border_style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(" Metrics ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let m = &self.state.metrics;
        let inner = block.inner(area);
        block.render(area, buf);

        let gc_color = if m.gc_ratio > 0.2 {
            Color::Red
        } else if m.gc_ratio > 0.1 {
            Color::Yellow
        } else {
            Color::Green
        };

        let spill_color = if m.spill_bytes > 1_000_000_000 {
            Color::Red
        } else if m.spill_bytes > 100_000_000 {
            Color::Yellow
        } else {
            Color::White
        };

        let spill_warn = if m.spill_bytes > 100_000_000 { " ⚠" } else { "" };
        let gc_warn = if m.gc_ratio > 0.2 { " ⚠" } else { "" };
        let skew_warn = if m.skew_stages > 0 { " ⚠" } else { "" };

        let lines: Vec<Line> = vec![
            Line::from(vec![
                Span::styled("Input:    ", Style::default().fg(Color::Gray)),
                Span::raw(format_bytes(m.input_bytes)),
            ]),
            Line::from(vec![
                Span::styled("Spill:    ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}{}", format_bytes(m.spill_bytes), spill_warn),
                    Style::default().fg(spill_color),
                ),
            ]),
            Line::from(vec![
                Span::styled("Shfl Read:", Style::default().fg(Color::Gray)),
                Span::raw(format_bytes(m.shuffle_read_bytes)),
            ]),
            Line::from(vec![
                Span::styled("Shfl Write:", Style::default().fg(Color::Gray)),
                Span::raw(format_bytes(m.shuffle_write_bytes)),
            ]),
            Line::from(vec![
                Span::styled("GC Ratio: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.1}%{}", m.gc_ratio * 100.0, gc_warn),
                    Style::default().fg(gc_color),
                ),
            ]),
            Line::from(vec![
                Span::styled("Skew:     ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{} stages{}", m.skew_stages, skew_warn),
                    if m.skew_stages > 0 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Max Task: ", Style::default().fg(Color::Gray)),
                Span::raw(format_ms(m.max_task_ms)),
            ]),
            Line::from(vec![
                Span::styled("Avg Task: ", Style::default().fg(Color::Gray)),
                Span::raw(format_ms(m.avg_task_ms as i64)),
            ]),
            Line::from(vec![
                Span::styled("Tasks:    ", Style::default().fg(Color::Gray)),
                Span::raw(format!("{}", m.total_tasks)),
            ]),
            Line::from(vec![
                Span::styled("Failed:   ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", m.failed_tasks),
                    if m.failed_tasks > 0 {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default()
                    },
                ),
            ]),
        ];

        Paragraph::new(lines).render(inner, buf);
    }
}

fn format_bytes(bytes: i64) -> String {
    if bytes <= 0 {
        return "0 B".to_string();
    }
    let b = bytes as f64;
    if b < 1024.0 {
        format!("{:.0} B", b)
    } else if b < 1024.0 * 1024.0 {
        format!("{:.1} KB", b / 1024.0)
    } else if b < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB", b / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", b / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_ms(ms: i64) -> String {
    if ms <= 0 {
        return "--".to_string();
    }
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}m{}s", ms / 60_000, (ms % 60_000) / 1000)
    }
}
