use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};

use crate::commands::claude_task::{get_git_project_info, GitProjectInfo, TaskManager, TaskStatus};

/// Map task status to display color
fn status_color(status: TaskStatus) -> Color {
    match status {
        TaskStatus::InProgress => Color::Blue,
        TaskStatus::Stop => Color::Yellow,
        TaskStatus::SessionEnded => Color::Gray,
        TaskStatus::Error => Color::Red,
    }
}

/// Application state
struct App {
    task_manager: TaskManager,
    selected_index: usize,
    list_state: ListState,
    should_quit: bool,
    last_refresh: Instant,
    auto_refresh_interval: Duration,
    /// Cache for git project info (worktree_path -> info)
    git_info_cache: HashMap<String, GitProjectInfo>,
}

impl App {
    fn new() -> Result<Self> {
        // Load Claude Code task progress
        let task_manager = match TaskManager::load() {
            Ok(tm) => tm,
            Err(e) => {
                eprintln!("Warning: Failed to load Claude Code task progress: {}", e);
                eprintln!("    Continuing with empty task manager...");
                TaskManager::default()
            }
        };

        let mut list_state = ListState::default();
        let tasks = task_manager.latest_tasks_by_worktree();
        if !tasks.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            task_manager,
            selected_index: 0,
            list_state,
            should_quit: false,
            last_refresh: Instant::now(),
            auto_refresh_interval: Duration::from_secs(1), // Auto-refresh every 1 second
            git_info_cache: HashMap::new(),
        })
    }

    fn next(&mut self) {
        let tasks = self.task_manager.latest_tasks_by_worktree();
        if tasks.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= tasks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
    }

    fn previous(&mut self) {
        let tasks = self.task_manager.latest_tasks_by_worktree();
        if tasks.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    tasks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
    }

    fn refresh(&mut self) -> Result<()> {
        // Refresh Claude Code task progress (only reloads changed files)
        if let Err(e) = self.task_manager.refresh() {
            eprintln!(
                "Warning: Failed to refresh Claude Code task progress: {}",
                e
            );
        }

        // Maintain selection state
        let tasks = self.task_manager.latest_tasks_by_worktree();
        if self.selected_index >= tasks.len() {
            self.selected_index = tasks.len().saturating_sub(1);
        }
        if !tasks.is_empty() {
            self.list_state.select(Some(self.selected_index));
        }

        // Update last refresh time
        self.last_refresh = Instant::now();

        Ok(())
    }

    /// Check if auto-refresh is needed and perform it
    fn try_auto_refresh(&mut self) -> Result<()> {
        if self.last_refresh.elapsed() >= self.auto_refresh_interval {
            self.refresh()?;
        }
        Ok(())
    }

    /// Get project display name for a worktree path, using cache
    fn get_project_name(&mut self, worktree_path: &str) -> String {
        if !self.git_info_cache.contains_key(worktree_path) {
            let info = get_git_project_info(worktree_path);
            self.git_info_cache.insert(worktree_path.to_string(), info);
        }
        self.git_info_cache
            .get(worktree_path)
            .map(|info| info.display_name())
            .unwrap_or_else(|| worktree_path.to_string())
    }
}

/// Execute UI command
pub fn execute() -> Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run application
    let app = App::new()?;
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.previous();
                    }
                    KeyCode::Char('r') => {
                        app.refresh()?;
                    }
                    _ => {}
                }
            }
        }

        // Auto-refresh at regular intervals
        app.try_auto_refresh()?;

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Task list
            Constraint::Length(9), // Task details (6 lines + border)
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new("ccmon - Claude Code Monitor")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Task list
    render_task_list(f, app, chunks[1]);

    // Task details
    render_task_details(f, app, chunks[2]);

    // Footer
    let active_tasks = app.task_manager.active_tasks().len();
    let total_tasks = app.task_manager.latest_tasks_by_worktree().len();
    let footer_text = format!(
        "Active: {} | Total: {} | Press r to refresh, q to quit",
        active_tasks, total_tasks
    );
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[3]);
}

fn render_task_list(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let tasks = app.task_manager.latest_tasks_by_worktree();

    if tasks.is_empty() {
        let empty = Paragraph::new(
            "No Claude Code tasks found\n\nMake sure hooks are initialized with: ccmon init",
        )
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Claude Code Tasks"),
        );
        f.render_widget(empty, area);
        return;
    }

    // Collect worktree paths first to release the borrow on app
    let worktree_paths: Vec<String> = tasks.iter().map(|t| t.worktree_path.clone()).collect();
    drop(tasks);

    // Pre-compute display names (populates cache)
    let display_names: Vec<String> = worktree_paths
        .iter()
        .map(|path| app.get_project_name(path))
        .collect();

    // Re-borrow tasks for rendering
    let tasks = app.task_manager.latest_tasks_by_worktree();
    let items: Vec<ListItem> = tasks
        .iter()
        .zip(display_names.iter())
        .map(|(task, display_name)| {
            let color = status_color(task.status);

            let line = Line::from(vec![
                Span::raw(format!("{} ", task.status.emoji())),
                Span::styled(format!("{:28}", display_name), Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!(" {:15}", task.status.description()),
                    Style::default().fg(color),
                ),
                Span::styled(
                    format!(" {}", task.duration_string()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Claude Code Tasks (j/k to navigate)"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_task_details(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let tasks = app.task_manager.latest_tasks_by_worktree();

    let Some(task) = tasks.get(app.selected_index) else {
        let empty = Paragraph::new("No task selected")
            .block(Block::default().borders(Borders::ALL).title("Task Details"));
        f.render_widget(empty, area);
        return;
    };

    let color = status_color(task.status);

    // Get project display name from cache (populated by render_task_list)
    let project_name = app
        .git_info_cache
        .get(&task.worktree_path)
        .map(|info| info.display_name())
        .unwrap_or_else(|| task.worktree_path.clone());

    let detail_lines = vec![
        Line::from(vec![
            Span::styled("Project: ", Style::default().fg(Color::Cyan)),
            Span::raw(project_name),
        ]),
        Line::from(vec![
            Span::styled("Session: ", Style::default().fg(Color::Cyan)),
            Span::raw(&task.session_id[..12.min(task.session_id.len())]),
        ]),
        Line::from(vec![
            Span::styled("Directory: ", Style::default().fg(Color::Cyan)),
            Span::raw(&task.worktree_path),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Cyan)),
            Span::styled(task.status.description(), Style::default().fg(color)),
        ]),
        Line::from(vec![
            Span::styled("Duration: ", Style::default().fg(Color::Cyan)),
            Span::raw(task.duration_string()),
        ]),
        Line::from(vec![
            Span::styled("Last activity: ", Style::default().fg(Color::Cyan)),
            Span::raw(&task.last_message),
        ]),
    ];

    let detail = Paragraph::new(detail_lines)
        .block(Block::default().borders(Borders::ALL).title("Task Details"));
    f.render_widget(detail, area);
}
