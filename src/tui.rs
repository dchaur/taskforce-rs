use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use crate::{TODO_FILE, TodoList};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub struct App {
    pub todo: TodoList,
    pub list_state: ListState,
    pub should_quit: bool,
    pub confirm_delete: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            todo: TodoList::load(TODO_FILE),
            list_state,
            should_quit: false,
            confirm_delete: None,
        }
    }
}

pub fn run_tui_mode() -> Result<(), String> {
    enable_raw_mode().map_err(|e| e.to_string())?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| e.to_string())?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| e.to_string())?;

    let mut app = App::new();

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode().map_err(|e| e.to_string())?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|e| e.to_string())?;
    terminal.show_cursor().map_err(|e| e.to_string())?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), String> {
    loop {
        terminal.draw(|f| ui(f, app)).map_err(|e| e.to_string())?;

        if let Event::Key(key) = event::read().map_err(|e| e.to_string())? {
            if key.kind == KeyEventKind::Press {
                handle_key_event(key, app)?;
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn get_selected_task(app: &App) -> Option<(String, bool)> {
    let selected_idx = app.list_state.selected()?;

    let (todo_items, done_items) = app.todo.list();
    let todo_vec: Vec<_> = todo_items.collect();
    let done_vec: Vec<_> = done_items.collect();

    let total_todo = todo_vec.len();

    if selected_idx < total_todo {
        return Some((todo_vec[selected_idx].name.clone(), false));
    }

    let done_idx = selected_idx - total_todo;

    if done_idx < done_vec.len() {
        return Some((done_vec[done_idx].name.clone(), true));
    }

    None
}

fn handle_key_event(key: KeyEvent, app: &mut App) -> Result<(), String> {
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }

        KeyCode::Down | KeyCode::Char('j') => {
            let i = match app.list_state.selected() {
                Some(i) => {
                    let todo_count = app.todo.len();

                    if todo_count > 0 && i < todo_count - 1 {
                        i + 1
                    } else {
                        i
                    }
                }
                None => 0,
            };
            app.list_state.select(Some(i));
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let i = match app.list_state.selected() {
                Some(i) => {
                    if i > 0 {
                        i - 1
                    } else {
                        0
                    }
                }
                None => 0,
            };
            app.list_state.select(Some(i));
        }
        KeyCode::Char('d') => {
            if let Some((task_name, is_done)) = get_selected_task(app) {
                let command = if is_done { "mark-todo" } else { "mark-done" };

                let _ = crate::handle_command(
                    &mut app.todo,
                    command,
                    Some(vec![task_name]),
                    None,
                    None,
                    None,
                    None,
                    crate::TODO_FILE,
                );
            }
        }
        KeyCode::Char('x') => {
            if app.confirm_delete.is_some() {
                app.confirm_delete = None;

                return Ok(());
            }

            if let Some((task_name, _)) = get_selected_task(app) {
                app.confirm_delete = Some(task_name);
            }
        }
        KeyCode::Char('y') => {
            if let Some(task_name) = app.confirm_delete.take() {
                let _ = crate::handle_command(
                    &mut app.todo,
                    "remove",
                    Some(vec![task_name]),
                    None,
                    None,
                    None,
                    None,
                    crate::TODO_FILE,
                );

                let new_len = app.todo.len();

                if new_len == 0 {
                    app.list_state.select(None);
                }else if let Some(selected) = app.list_state.selected() {
                    if selected >= new_len {
                        app.list_state.select(Some(new_len - 1));
                    }
                }
            }
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.confirm_delete = None;
        }
        _ => {}
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    let title = Paragraph::new("📋 Todo CLI - TUI Mode")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let tasks: Vec<ListItem> = {
        let (todo_items, done_items) = app.todo.list();

        let todo_vec: Vec<_> = todo_items.map(|item| (item, crate::Status::Todo)).collect();

        let done_vec: Vec<_> = done_items.map(|item| (item, crate::Status::Done)).collect();

        todo_vec
            .into_iter()
            .chain(done_vec)
            .map(|(item, status)| {
                let status_icon = if status == crate::Status::Done {
                    "✓"
                } else {
                    " "
                };
                let priority_icon = match item.priority {
                    crate::Priority::High => "⬆️",
                    crate::Priority::Medium => "➡️",
                    crate::Priority::Low => "⬇️",
                };

                let content = format!("[{}] {} {}", status_icon, priority_icon, item.name);
                
                let style = if status == crate::Status::Done {
                    Style::default().fg(Color::LightGreen).add_modifier(Modifier::DIM)
                } else {
                    match item.priority {
                        crate::Priority::High => Style::default().fg(Color::Red),
                        crate::Priority::Medium => Style::default().fg(Color::Yellow),
                        crate::Priority::Low => Style::default().fg(Color::Green),
                    }
                };
                
                
                ListItem::new(content).style(style)
            })
            .collect()
    };

    let tasks_list = List::new(tasks)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Tasks (↑↓/jk to navigate, q to quit)"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(tasks_list, chunks[1], &mut app.list_state.clone());

    let help_text = if let Some(ref task_name) = app.confirm_delete {
        format!(
            "Delete '{}'? Press 'y' to confirm, 'n' or ESC to cancel",
            task_name
        )
    } else {
        "'q' quit | '↑↓/jk' navigate | 'd' toggle | 'x' delete".to_string()
    };

    let help = Paragraph::new(help_text)
        .style(if app.confirm_delete.is_some() {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        })
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}
