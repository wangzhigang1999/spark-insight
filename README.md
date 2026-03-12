# spark-insight

Rust 编写的 Spark Eventlog 智能分析工具。将 JSON Lines 格式的 eventlog 转换为 Parquet 列式存储，通过 DataFusion SQL 引擎查询，并以 ratatui TUI 仪表盘展示分析结果。

## 功能特性

- **自动缓存**：首次解析后写入 Parquet（`~/.spark-insight/cache/`），二次打开毫秒级加载
- **TUI 仪表盘**：Job/Stage 树形导航、指标面板、Task 倾斜分布条、内置 SQL REPL
- **内置诊断**：数据倾斜、内存 Spill、Shuffle 放大、GC 压力一键分析
- **SQL 自由查询**：DataFusion 引擎，直接对 `tasks / stages / jobs / executors` 四张表执行任意 SQL
- **非交互模式**：适合脚本和 CI，支持表格或 JSON 输出

## 安装

```bash
cargo install --path .
```

需要 Rust 1.75+。

## 快速开始

```bash
# 打开 TUI 仪表盘（主要用法）
spark-insight app-20240101.log

# 内置诊断报告
spark-insight analyze app-20240101.log

# 聚焦特定问题
spark-insight analyze app-20240101.log --focus skew,spill --top 10

# 输出 JSON（适合管道/CI）
spark-insight analyze app-20240101.log --output json | jq .

# 执行单条 SQL
spark-insight sql app-20240101.log \
  "SELECT stage_id, MAX(duration_ms) FROM tasks GROUP BY 1 ORDER BY 2 DESC LIMIT 10"
```

## TUI 仪表盘

```
┌─ spark-insight ── app-20240101.log ─────────────────────────────┐
│ [App] spark-example-etl    Duration: 3m42s   Tasks: 12843       │
├─ Jobs/Stages (↑↓ 导航) ─────┬─ Metrics ────────────────────────┤
│ ▼ Job 0  (3 stages) 41s     │ Input:    1.0 GB                  │
│   ├ Stage  1  29s  ★SKEW    │ Spill:    2.5 GB ⚠                │
│   ├ Stage  0   6s            │ Shfl Read: 565 MB                 │
│   └ Stage  2   3s            │ GC Ratio:  29.7% ⚠               │
├─ Task Distribution ─────────┴──────────────────────────────────┤
│ Stage 1 [  256]  ████████████░░░░  max/med=    1x               │
│ Stage 0 [  128]  ███░░░░░░░░░░░░░  max/med=    1x               │
│ Stage 2 [  512]  █░░░░░░░░░░░░░░░  max/med=    1x               │
├─ SQL ──────────────────────────────────────────────────────────┤
│ spark> SELECT stage_id, SUM(disk_spill_bytes) FROM tasks ...   │
│ [Tab:切换面板] [↑↓:导航] [Enter:展开] [F5:执行SQL] [q:退出]    │
└───────────────────────────────────────────────────────────────┘
```

### 键盘操作

| 按键 | 功能 |
|------|------|
| `Tab` | 切换激活面板 |
| `↑` / `↓` | 列表导航 |
| `Enter` | 展开 Job / 钻取 Stage |
| `F5` / `Ctrl+Enter` | 执行 SQL |
| `Ctrl+↑` / `Ctrl+↓` | SQL 历史记录 |
| `Esc` | 清空 SQL 输入框 |
| `1` / `2` / `3` | 快速跳转面板 |
| `?` | 显示帮助 |
| `q` / `Ctrl+C` | 退出 |

## 数据表结构

| 表名 | 说明 |
|------|------|
| `tasks` | 每个 Task 一行，含耗时、输入输出、Shuffle、Spill、GC 指标 |
| `stages` | 每个 Stage 一行，含提交/完成时间、Task 数量 |
| `jobs` | 每个 Job 一行，含状态、耗时、失败原因 |
| `executors` | 每个 Executor 一行，含核数、存活时间 |

## 缓存管理

```bash
# 查看所有缓存
spark-insight cache list

# 清除指定 log 的缓存
spark-insight cache clear app-20240101.log

# 清除全部缓存
spark-insight cache clear-all
```

缓存存储在 `~/.spark-insight/cache/`，以文件名 + 大小 + 修改时间为 key，文件更新后自动失效。

## 项目结构

```
src/
├── main.rs              # tokio async main，CLI 路由
├── cli.rs               # clap v4 命令定义
├── error.rs             # thiserror 错误类型
├── parser/              # 流式 JSON Lines 解析（serde）
├── store/               # Arrow Schema + RecordBatch 构建 + Parquet 缓存
├── query/               # DataFusion SessionContext + 内置诊断 SQL
├── tui/                 # ratatui TUI 主循环、状态机、各面板 widget
└── reporter/            # 非交互模式：comfy-table 表格 + JSON 输出
```

## 主要依赖

| Crate | 用途 |
|-------|------|
| `arrow` + `parquet` | 列式存储 |
| `datafusion` | SQL 查询引擎 |
| `ratatui` + `crossterm` | TUI 渲染 |
| `tui-input` | SQL 输入框 |
| `serde_json` | JSON 解析 |
| `clap` | CLI |
| `tokio` | 异步运行时 |

## 路线图

- [ ] 阶段二：LLM Agent + ReAct 模式（Claude API），自然语言问题 → 自动生成 SQL → 解读结果
- [ ] 多 eventlog 对比分析
- [ ] release binary（GitHub Actions）
