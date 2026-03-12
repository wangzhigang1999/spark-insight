use crate::tui::app::AppState;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct SkewBarWidget<'a> {
    pub state: &'a AppState,
}

impl<'a> Widget for SkewBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Task Distribution ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.state.skew_bars.is_empty() {
            Paragraph::new("No task data available").render(inner, buf);
            return;
        }

        // Available width for the bar itself
        let bar_width = inner.width.saturating_sub(40) as usize;
        if bar_width == 0 {
            return;
        }

        let max_task_ms = self
            .state
            .skew_bars
            .iter()
            .map(|b| b.max_ms)
            .max()
            .unwrap_or(1)
            .max(1);

        let lines: Vec<Line> = self
            .state
            .skew_bars
            .iter()
            .take(inner.height as usize)
            .map(|bar| {
                let fill = if max_task_ms > 0 {
                    (bar.max_ms as f64 / max_task_ms as f64 * bar_width as f64) as usize
                } else {
                    0
                };
                let median_fill = if max_task_ms > 0 {
                    (bar.median_ms as f64 / max_task_ms as f64 * bar_width as f64) as usize
                } else {
                    0
                };

                let bar_str: String = (0..bar_width)
                    .map(|i| {
                        if i < median_fill {
                            '█'
                        } else if i < fill {
                            '░'
                        } else {
                            ' '
                        }
                    })
                    .collect();

                let skew_color = if bar.skew_ratio > 100.0 {
                    Color::Red
                } else if bar.skew_ratio > 10.0 {
                    Color::Yellow
                } else {
                    Color::Green
                };

                let skew_warn = if bar.skew_ratio > 10.0 { " ⚠" } else { "  " };

                Line::from(vec![
                    Span::styled(
                        format!("Stage {:3} [{:5}] ", bar.stage_id, bar.task_count),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(bar_str, Style::default().fg(skew_color)),
                    Span::styled(
                        format!(" max/med={:5.0}x{}", bar.skew_ratio, skew_warn),
                        Style::default().fg(skew_color),
                    ),
                ])
            })
            .collect();

        Paragraph::new(lines).render(inner, buf);
    }
}
