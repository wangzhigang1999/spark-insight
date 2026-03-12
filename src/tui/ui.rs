use crate::tui::app::AppState;
use crate::tui::widgets::{
    metrics_panel::MetricsPanelWidget, skew_bar::SkewBarWidget, sql_panel::SqlPanelWidget,
    stage_tree::StageTreeWidget,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Frame,
};

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    render_layout(frame.buffer_mut(), area, state);

    if state.show_help {
        render_help_overlay(frame.buffer_mut(), area);
    }
}

fn render_layout(buf: &mut Buffer, area: Rect, state: &AppState) {
    // Title bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Min(10),   // main area
            Constraint::Length(1), // status bar
        ])
        .split(area);

    render_title_bar(buf, chunks[0], state);
    render_main_area(buf, chunks[1], state);
    render_status_bar(buf, chunks[2]);
}

fn render_title_bar(buf: &mut Buffer, area: Rect, state: &AppState) {
    let app_info = &state.app_info;
    let duration_ms = if app_info.end_time > app_info.start_time {
        app_info.end_time - app_info.start_time
    } else {
        0
    };
    let duration_str = format_duration(duration_ms);
    let cache_str = if state.from_cache { " (cached)" } else { "" };

    let title = format!(
        " spark-insight ── {} ── App: {}  Duration: {}  {}",
        state.log_file_name, app_info.app_name, duration_str, cache_str
    );

    Paragraph::new(title)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .render(area, buf);
}

fn render_main_area(buf: &mut Buffer, area: Rect, state: &AppState) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45), // top row: tree + metrics
            Constraint::Percentage(20), // skew bars
            Constraint::Percentage(35), // sql panel
        ])
        .split(area);

    // Top row: stage tree (left) + metrics (right)
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[0]);

    StageTreeWidget { state }.render(top_chunks[0], buf);
    MetricsPanelWidget { state }.render(top_chunks[1], buf);

    // Skew bars
    SkewBarWidget { state }.render(main_chunks[1], buf);

    // SQL panel
    SqlPanelWidget { state }.render(main_chunks[2], buf);
}

fn render_status_bar(buf: &mut Buffer, area: Rect) {
    let status = " [Tab] Switch Panel  [↑↓] Navigate  [Enter] Expand/Drill  [F5] Execute SQL  [Ctrl+↑] SQL History  [?] Help  [q] Quit";
    Paragraph::new(status)
        .style(Style::default().fg(Color::DarkGray))
        .render(area, buf);
}

fn render_help_overlay(buf: &mut Buffer, area: Rect) {
    let width = 60u16.min(area.width.saturating_sub(4));
    let height = 22u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let overlay_area = Rect::new(x, y, width, height);

    Clear.render(overlay_area, buf);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Keyboard Shortcuts",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  Tab          Switch active panel"),
        Line::from("  ↑ / ↓        Navigate list / tree"),
        Line::from("  Enter        Expand Job / drill Stage"),
        Line::from("  1 / 2 / 3    Jump to panel"),
        Line::from("  F5           Execute SQL"),
        Line::from("  Ctrl+Enter   Execute SQL (alt)"),
        Line::from("  Ctrl+↑       Previous SQL history"),
        Line::from("  Ctrl+↓       Next SQL history"),
        Line::from("  Esc          Clear SQL input"),
        Line::from("  ?            Toggle this help"),
        Line::from("  q / Ctrl+C   Quit"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Built-in Tables",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  tasks    stages    jobs    executors"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Press ? to close",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .render(overlay_area, buf);
}

fn format_duration(ms: i64) -> String {
    if ms <= 0 {
        return "--".to_string();
    }
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else if ms < 3_600_000 {
        format!("{}m{}s", ms / 60_000, (ms % 60_000) / 1000)
    } else {
        format!("{}h{}m{}s", ms / 3_600_000, (ms % 3_600_000) / 60_000, (ms % 60_000) / 1000)
    }
}
