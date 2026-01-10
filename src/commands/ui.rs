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
use std::io;
use std::time::{Duration, Instant};

use crate::commands::claude_task::{TaskManager, TaskStatus};

/// Application state
struct App {
    task_manager: TaskManager,
    selected_index: usize,
    list_state: ListState,
    should_quit: bool,
    last_refresh: Instant,
    auto_refresh_interval: Duration,
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
        let tasks = task_manager.all_tasks();
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
        })
    }

    fn next(&mut self) {
        let tasks = self.task_manager.all_tasks();
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
        let tasks = self.task_manager.all_tasks();
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
            eprintln!("Warning: Failed to refresh Claude Code task progress: {}", e);
        }

        // Maintain selection state
        let tasks = self.task_manager.all_tasks();
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
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Task list
            Constraint::Length(8),  // Task details
            Constraint::Length(3),  // Footer
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
    let total_tasks = app.task_manager.all_tasks().len();
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
    let tasks = app.task_manager.all_tasks();

    if tasks.is_empty() {
        let empty = Paragraph::new("No Claude Code tasks found\n\nMake sure hooks are initialized with: ccmon init")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Claude Code Tasks"),
            );
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = tasks
        .iter()
        .map(|task| {
            let status_emoji = task.status.emoji();
            let status_text = match task.status {
                TaskStatus::InProgress => "In Progress",
                TaskStatus::Stop => "⚠ Stop",
                TaskStatus::SessionEnded => "Session Ended",
                TaskStatus::Error => "Error",
            };

            let color = match task.status {
                TaskStatus::InProgress => Color::Blue,
                TaskStatus::Stop => Color::Yellow,
                TaskStatus::SessionEnded => Color::Gray,
                TaskStatus::Error => Color::Red,
            };

            // Extract directory name from path
            let dir_name = std::path::Path::new(&task.worktree_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let line = Line::from(vec![
                Span::raw(format!("{} ", status_emoji)),
                Span::styled(format!("{:20}", dir_name), Style::default().fg(Color::Cyan)),
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
    let tasks = app.task_manager.all_tasks();

    if let Some(task) = tasks.get(app.selected_index) {
        let status_color = match task.status {
            TaskStatus::InProgress => Color::Blue,
            TaskStatus::Stop => Color::Yellow,
            TaskStatus::SessionEnded => Color::Gray,
            TaskStatus::Error => Color::Red,
        };

        let detail_lines = vec![
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
                Span::styled(task.status.description(), Style::default().fg(status_color)),
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

        let detail = Paragraph::new(detail_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Task Details"),
        );
        f.render_widget(detail, area);
    } else {
        let empty = Paragraph::new("No task selected")
            .block(Block::default().borders(Borders::ALL).title("Task Details"));
        f.render_widget(empty, area);
    }
}
