use crate::tui::app::{AppState, FocusedPanel, TreeItem};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};

pub struct StageTreeWidget<'a> {
    pub state: &'a AppState,
}

impl<'a> Widget for StageTreeWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let focused = self.state.focused_panel == FocusedPanel::StageTree;
        let border_style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(" Jobs/Stages (↑↓ navigate, Enter expand) ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let items: Vec<ListItem> = self
            .state
            .tree_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let selected = i == self.state.tree_selected;
                render_tree_item(item, selected)
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(self.state.tree_selected));

        StatefulWidget::render(
            List::new(items)
                .block(block)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD)),
            area,
            buf,
            &mut list_state,
        );
    }
}

fn render_tree_item(item: &TreeItem, _selected: bool) -> ListItem<'static> {
    let duration_str = format_duration(item.duration_ms);
    let status_icon = if item.status.contains("Succeeded") || item.status.contains("JobSucceeded") {
        ""
    } else if item.status.contains("Failed") {
        "✗"
    } else {
        "…"
    };

    let warn_spill = if item.spill_bytes > 100_000_000 { " ⚠SPILL" } else { "" };
    let warn_skew = if item.skew_ratio > 10.0 { " ★SKEW" } else { "" };

    if item.is_job {
        let expand_icon = if item.is_expanded { "▼" } else { "▶" };
        let text = format!(
            "{} Job {:3}  {:8}  {}{}{}",
            expand_icon, item.job_id, duration_str, status_icon, warn_spill, warn_skew
        );
        let style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        ListItem::new(Line::from(Span::styled(text, style)))
    } else {
        let text = format!(
            "  ├ Stage {:3}  {:8}  {:5} tasks{}{}",
            item.stage_id.unwrap_or(-1),
            duration_str,
            item.num_tasks,
            warn_spill,
            warn_skew
        );
        let style = if item.skew_ratio > 10.0 {
            Style::default().fg(Color::Red)
        } else if item.spill_bytes > 0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        ListItem::new(Line::from(Span::styled(text, style)))
    }
}

fn format_duration(ms: i64) -> String {
    if ms <= 0 {
        return "  --   ".to_string();
    }
    if ms < 1000 {
        format!("{:4}ms", ms)
    } else if ms < 60_000 {
        format!("{:5.1}s", ms as f64 / 1000.0)
    } else if ms < 3_600_000 {
        format!("{:3}m{:02}s", ms / 60_000, (ms % 60_000) / 1000)
    } else {
        format!("{:2}h{:02}m", ms / 3_600_000, (ms % 3_600_000) / 60_000)
    }
}
