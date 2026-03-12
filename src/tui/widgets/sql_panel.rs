use crate::tui::app::{AppState, FocusedPanel};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Widget},
};

pub struct SqlPanelWidget<'a> {
    pub state: &'a AppState,
}

impl<'a> Widget for SqlPanelWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let focused = self.state.focused_panel == FocusedPanel::SqlInput
            || self.state.focused_panel == FocusedPanel::SqlResult;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        // Input line
        let input_border_style = if self.state.focused_panel == FocusedPanel::SqlInput {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let cursor_indicator = if self.state.sql_executing {
            "⏳ "
        } else {
            "spark> "
        };

        let input_text = format!(
            "{}{}",
            cursor_indicator,
            self.state.sql_input.value()
        );

        Paragraph::new(input_text)
            .block(
                Block::default()
                    .title(" SQL (F5/Ctrl+Enter to execute, Ctrl+↑ history) ")
                    .borders(Borders::ALL)
                    .border_style(input_border_style),
            )
            .render(chunks[0], buf);

        // Result area
        let result_border_style = if self.state.focused_panel == FocusedPanel::SqlResult {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        if let Some(err) = &self.state.sql_error {
            Paragraph::new(format!("Error: {}", err))
                .style(Style::default().fg(Color::Red))
                .block(
                    Block::default()
                        .title(" Result ")
                        .borders(Borders::ALL)
                        .border_style(result_border_style),
                )
                .render(chunks[1], buf);
        } else if let Some(table_data) = &self.state.sql_result {
            render_table(table_data, chunks[1], buf, result_border_style);
        } else {
            Paragraph::new("Execute a SQL query to see results here.\nBuilt-in tables: tasks, stages, jobs, executors")
                .style(Style::default().fg(Color::DarkGray))
                .block(
                    Block::default()
                        .title(" Result ")
                        .borders(Borders::ALL)
                        .border_style(result_border_style),
                )
                .render(chunks[1], buf);
        }
    }
}

fn render_table(
    data: &crate::query::format::TableData,
    area: Rect,
    buf: &mut Buffer,
    border_style: Style,
) {
    let header_cells: Vec<Cell> = data
        .headers
        .iter()
        .map(|h| {
            Cell::from(h.as_str()).style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        })
        .collect();

    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = data
        .rows
        .iter()
        .map(|row| {
            let cells: Vec<Cell> = row.iter().map(|c| Cell::from(c.as_str())).collect();
            Row::new(cells).height(1)
        })
        .collect();

    // Compute column widths
    let num_cols = data.headers.len();
    let widths: Vec<Constraint> = if num_cols == 0 {
        vec![]
    } else {
        let col_width = (area.width.saturating_sub(num_cols as u16 + 1)) / num_cols as u16;
        (0..num_cols)
            .map(|_| Constraint::Length(col_width.max(8)))
            .collect()
    };

    let row_count = data.rows.len();
    let title = format!(" Result ({} rows) ", row_count);

    Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .render(area, buf);
}
