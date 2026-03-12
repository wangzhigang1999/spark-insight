use crate::parser::types::AppInfo;
use crate::query::format::TableData;

/// Which panel is currently focused
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusedPanel {
    StageTree,
    Metrics,
    SqlInput,
    SqlResult,
}

/// Tree item in the job/stage list
#[derive(Debug, Clone)]
pub struct TreeItem {
    pub job_id: i32,
    pub stage_id: Option<i32>,
    pub label: String,
    pub duration_ms: i64,
    pub is_expanded: bool,
    pub is_job: bool,
    pub num_tasks: i32,
    pub status: String,
    pub spill_bytes: i64,
    pub skew_ratio: f64,
}

/// Metrics for the selected stage/job
#[derive(Debug, Clone, Default)]
pub struct MetricsDisplay {
    pub input_bytes: i64,
    pub spill_bytes: i64,
    pub shuffle_read_bytes: i64,
    pub shuffle_write_bytes: i64,
    pub gc_ratio: f64,
    pub skew_stages: usize,
    pub max_task_ms: i64,
    pub avg_task_ms: f64,
    pub total_tasks: i32,
    pub failed_tasks: i32,
}

/// Skew bar data for a single stage
#[derive(Debug, Clone)]
pub struct SkewBarItem {
    pub stage_id: i32,
    pub task_count: i32,
    pub max_ms: i64,
    pub median_ms: i64,
    pub skew_ratio: f64,
}

/// Main application state
pub struct AppState {
    pub app_info: AppInfo,
    pub log_file_name: String,
    pub from_cache: bool,

    // Stage tree
    pub tree_items: Vec<TreeItem>,
    pub tree_selected: usize,

    // Metrics panel (updates based on selection)
    pub metrics: MetricsDisplay,

    // Skew bars
    pub skew_bars: Vec<SkewBarItem>,

    // SQL panel
    pub sql_input: tui_input::Input,
    pub sql_history: Vec<String>,
    pub sql_history_idx: Option<usize>,
    pub sql_result: Option<TableData>,
    pub sql_error: Option<String>,
    pub sql_executing: bool,

    // Focus
    pub focused_panel: FocusedPanel,

    // Help overlay
    pub show_help: bool,

    // Should quit
    pub should_quit: bool,
}

impl AppState {
    pub fn new(app_info: AppInfo, log_file_name: String, from_cache: bool) -> Self {
        AppState {
            app_info,
            log_file_name,
            from_cache,
            tree_items: vec![],
            tree_selected: 0,
            metrics: MetricsDisplay::default(),
            skew_bars: vec![],
            sql_input: tui_input::Input::default(),
            sql_history: vec![],
            sql_history_idx: None,
            sql_result: None,
            sql_error: None,
            sql_executing: false,
            focused_panel: FocusedPanel::StageTree,
            show_help: false,
            should_quit: false,
        }
    }

    pub fn selected_stage_id(&self) -> Option<i32> {
        self.tree_items
            .get(self.tree_selected)
            .and_then(|item| item.stage_id)
    }

    pub fn selected_job_id(&self) -> Option<i32> {
        self.tree_items
            .get(self.tree_selected)
            .map(|item| item.job_id)
    }

    pub fn navigate_up(&mut self) {
        if self.tree_selected > 0 {
            self.tree_selected -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if self.tree_selected + 1 < self.tree_items.len() {
            self.tree_selected += 1;
        }
    }

    pub fn toggle_expand(&mut self) {
        if let Some(item) = self.tree_items.get_mut(self.tree_selected) {
            if item.is_job {
                item.is_expanded = !item.is_expanded;
            }
        }
    }

    pub fn sql_history_prev(&mut self) {
        if self.sql_history.is_empty() {
            return;
        }
        let idx = match self.sql_history_idx {
            None => self.sql_history.len() - 1,
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
        };
        self.sql_history_idx = Some(idx);
        let sql = self.sql_history[idx].clone();
        self.sql_input = tui_input::Input::new(sql);
    }

    pub fn sql_history_next(&mut self) {
        if let Some(idx) = self.sql_history_idx {
            if idx + 1 < self.sql_history.len() {
                let next = idx + 1;
                self.sql_history_idx = Some(next);
                let sql = self.sql_history[next].clone();
                self.sql_input = tui_input::Input::new(sql);
            } else {
                self.sql_history_idx = None;
                self.sql_input = tui_input::Input::default();
            }
        }
    }

    pub fn push_sql_history(&mut self, sql: String) {
        // Avoid duplicates at end
        if self.sql_history.last().map(|s| s.as_str()) != Some(&sql) {
            self.sql_history.push(sql);
        }
        self.sql_history_idx = None;
    }
}
