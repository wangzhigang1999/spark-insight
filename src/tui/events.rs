use crate::tui::app::{AppState, FocusedPanel};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use tui_input::backend::crossterm::EventHandler;

/// Returns true if the SQL should be executed
pub fn handle_event(event: Event, state: &mut AppState) -> bool {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            handle_key(key, state)
        }
        _ => false,
    }
}

fn handle_key(key: crossterm::event::KeyEvent, state: &mut AppState) -> bool {
    // Global keys
    match key.code {
        KeyCode::Char('q') if state.focused_panel != FocusedPanel::SqlInput => {
            state.should_quit = true;
            return false;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.should_quit = true;
            return false;
        }
        KeyCode::Char('?') => {
            state.show_help = !state.show_help;
            return false;
        }
        KeyCode::Esc => {
            if state.show_help {
                state.show_help = false;
            } else if state.focused_panel == FocusedPanel::SqlInput {
                state.sql_input = tui_input::Input::default();
                state.sql_history_idx = None;
            }
            return false;
        }
        KeyCode::Tab => {
            cycle_panel(state);
            return false;
        }
        KeyCode::Char('1') if state.focused_panel != FocusedPanel::SqlInput => {
            state.focused_panel = FocusedPanel::StageTree;
            return false;
        }
        KeyCode::Char('2') if state.focused_panel != FocusedPanel::SqlInput => {
            state.focused_panel = FocusedPanel::Metrics;
            return false;
        }
        KeyCode::Char('3') if state.focused_panel != FocusedPanel::SqlInput => {
            state.focused_panel = FocusedPanel::SqlInput;
            return false;
        }
        _ => {}
    }

    // Panel-specific keys
    match &state.focused_panel {
        FocusedPanel::StageTree => handle_tree_keys(key, state),
        FocusedPanel::SqlInput => handle_sql_keys(key, state),
        FocusedPanel::SqlResult => handle_result_keys(key, state),
        FocusedPanel::Metrics => false,
    }
}

fn handle_tree_keys(key: crossterm::event::KeyEvent, state: &mut AppState) -> bool {
    match key.code {
        KeyCode::Up => state.navigate_up(),
        KeyCode::Down => state.navigate_down(),
        KeyCode::Enter => state.toggle_expand(),
        _ => {}
    }
    false
}

fn handle_sql_keys(key: crossterm::event::KeyEvent, state: &mut AppState) -> bool {
    match key.code {
        KeyCode::F(5) => {
            let sql = state.sql_input.value().trim().to_string();
            if !sql.is_empty() {
                state.push_sql_history(sql);
                return true; // Signal to execute
            }
        }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let sql = state.sql_input.value().trim().to_string();
            if !sql.is_empty() {
                state.push_sql_history(sql);
                return true;
            }
        }
        KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.sql_history_prev();
        }
        KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.sql_history_next();
        }
        _ => {
            state
                .sql_input
                .handle_event(&crossterm::event::Event::Key(key));
        }
    }
    false
}

fn handle_result_keys(key: crossterm::event::KeyEvent, state: &mut AppState) -> bool {
    match key.code {
        KeyCode::Up | KeyCode::Down => {}
        _ => {}
    }
    false
}

fn cycle_panel(state: &mut AppState) {
    state.focused_panel = match state.focused_panel {
        FocusedPanel::StageTree => FocusedPanel::Metrics,
        FocusedPanel::Metrics => FocusedPanel::SqlInput,
        FocusedPanel::SqlInput => FocusedPanel::SqlResult,
        FocusedPanel::SqlResult => FocusedPanel::StageTree,
    };
}
