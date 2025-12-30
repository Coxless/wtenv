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

use crate::worktree::{self, info::WorktreeDetail, process::ProcessManager};

/// アプリケーションの状態
struct App {
    worktrees: Vec<WorktreeDetail>,
    process_manager: ProcessManager,
    selected_index: usize,
    list_state: ListState,
    should_quit: bool,
}

impl App {
    fn new() -> Result<Self> {
        let worktrees_info = worktree::list_worktrees()?;
        let mut worktrees = Vec::new();

        for wt in worktrees_info {
            if let Ok(detail) =
                WorktreeDetail::from_path(&wt.path, wt.branch, wt.commit, wt.is_main)
            {
                worktrees.push(detail);
            }
        }

        let repo_root = worktree::get_repo_root()?;
        let process_manager = ProcessManager::load(&repo_root)?;

        let mut list_state = ListState::default();
        if !worktrees.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            worktrees,
            process_manager,
            selected_index: 0,
            list_state,
            should_quit: false,
        })
    }

    fn next(&mut self) {
        if self.worktrees.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.worktrees.len() - 1 {
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
        if self.worktrees.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.worktrees.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
    }

    fn get_selected_worktree(&self) -> Option<&WorktreeDetail> {
        self.worktrees.get(self.selected_index)
    }

    fn refresh(&mut self) -> Result<()> {
        let worktrees_info = worktree::list_worktrees()?;
        let mut worktrees = Vec::new();

        for wt in worktrees_info {
            if let Ok(detail) =
                WorktreeDetail::from_path(&wt.path, wt.branch, wt.commit, wt.is_main)
            {
                worktrees.push(detail);
            }
        }

        self.worktrees = worktrees;
        let repo_root = worktree::get_repo_root()?;
        self.process_manager = ProcessManager::load(&repo_root)?;

        // 選択状態を維持
        if self.selected_index >= self.worktrees.len() {
            self.selected_index = self.worktrees.len().saturating_sub(1);
        }
        if !self.worktrees.is_empty() {
            self.list_state.select(Some(self.selected_index));
        }

        Ok(())
    }
}

/// UIコマンドの実行
pub fn execute() -> Result<()> {
    // ターミナル設定
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // アプリケーション実行
    let app = App::new()?;
    let res = run_app(&mut terminal, app);

    // ターミナル復元
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

        if event::poll(std::time::Duration::from_millis(100))? {
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
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(8),
            Constraint::Length(3),
        ])
        .split(f.area());

    // ヘッダー
    let header = Paragraph::new("wtenv - Worktree Control Center")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Worktreeリスト
    let items: Vec<ListItem> = app
        .worktrees
        .iter()
        .map(|wt| {
            let status_emoji = wt.status_emoji();
            let branch = wt.branch.as_deref().unwrap_or("(detached)");
            let status_text = wt.status_text();

            let line = Line::from(vec![
                Span::raw(format!("{} ", status_emoji)),
                Span::styled(format!("{:30}", branch), Style::default().fg(Color::Green)),
                Span::styled(
                    format!(" {}", status_text),
                    Style::default().fg(Color::Yellow),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Worktrees (↑/↓ or j/k to navigate, r to refresh, q to quit)"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

    // 選択されたworktreeの詳細
    if let Some(selected) = app.get_selected_worktree() {
        let detail_lines = vec![
            Line::from(vec![
                Span::styled("Branch: ", Style::default().fg(Color::Cyan)),
                Span::raw(selected.branch.as_deref().unwrap_or("(detached)")),
            ]),
            Line::from(vec![
                Span::styled("Path: ", Style::default().fg(Color::Cyan)),
                Span::raw(&selected.path),
            ]),
            Line::from(vec![
                Span::styled("Modified files: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{}", selected.modified_files),
                    if selected.modified_files > 0 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Green)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Ahead commits: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{}", selected.ahead_commits),
                    if selected.ahead_commits > 0 {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default()
                    },
                ),
            ]),
        ];

        let detail = Paragraph::new(detail_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Worktree Details"),
        );
        f.render_widget(detail, chunks[2]);
    } else {
        let empty = Paragraph::new("No worktree selected")
            .block(Block::default().borders(Borders::ALL).title("Details"));
        f.render_widget(empty, chunks[2]);
    }

    // プロセス情報
    let process_count = app.process_manager.running_processes().len();
    let footer_text = format!(
        "Total: {} worktrees | {} running processes",
        app.worktrees.len(),
        process_count
    );
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[3]);
}
