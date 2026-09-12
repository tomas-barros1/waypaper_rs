use crate::services::{cache::Cache, wallpaper_service::WallpaperService};
use base64::{Engine, engine::general_purpose::STANDARD};
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect, Size},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::{
    io::{self, Stdout, Write},
    path::PathBuf,
    process::Command,
    time::Duration,
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let result = run_loop(&mut stdout);
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    result
}

struct TuiApp {
    all_wallpapers: Vec<PathBuf>,
    wallpapers: Vec<PathBuf>,
    selected: usize,
    search: String,
    editing: bool,
    message: String,
}

impl TuiApp {
    fn new(cache: &Cache) -> Result<Self, String> {
        let folder = cache
            .folder
            .clone()
            .ok_or("no wallpaper folder cached; use the GTK app first")?;
        let all_wallpapers = WallpaperService::new(cache.clone()).wallpapers_in(&folder);
        let wallpapers = all_wallpapers.clone();
        Ok(Self {
            all_wallpapers,
            wallpapers,
            selected: 0,
            search: String::new(),
            editing: false,
            message: String::new(),
        })
    }
    fn apply_filter(&mut self) {
        let query = self.search.to_lowercase();
        self.wallpapers = self
            .all_wallpapers
            .iter()
            .filter(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_lowercase().contains(&query))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        self.selected = self.selected.min(self.wallpapers.len().saturating_sub(1));
    }
    fn selected_path(&self) -> Option<&PathBuf> {
        self.wallpapers.get(self.selected)
    }
    fn move_selection(&mut self, delta: isize) {
        if self.wallpapers.is_empty() {
            return;
        }
        let len = self.wallpapers.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(len) as usize;
    }
}

fn run_loop(stdout: &mut Stdout) -> Result<(), Box<dyn std::error::Error>> {
    let cache = Cache::load_default();
    let mut app = TuiApp::new(&cache).map_err(io::Error::other)?;
    let backend = CrosstermBackend::new(&mut *stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut redraw = true;
    loop {
        if redraw {
            let selected = app.selected_path().cloned();
            terminal.draw(|frame| draw(frame, &app, selected.as_ref()))?;
            if let Some(path) = selected.as_ref() {
                let preview = preview_rect(terminal.size()?);
                draw_preview(terminal.backend_mut(), path, preview)?;
            }
            redraw = false;
        }
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if handle_key(&mut app, key)? {
                    break;
                }
                redraw = true;
            }
        }
    }
    Ok(())
}

fn draw(frame: &mut ratatui::Frame, app: &TuiApp, selected: Option<&PathBuf>) {
    let areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(frame.area());
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(2),
        ])
        .split(areas[0]);
    let search_label = if app.editing {
        format!("Search: {}█", app.search)
    } else {
        format!("Search: {}", app.search)
    };
    frame.render_widget(
        Paragraph::new(search_label).block(
            Block::default()
                .borders(Borders::ALL)
                .title("/ search · arrows navigate · Enter apply"),
        ),
        left[0],
    );
    let columns = 4usize;
    let mut lines = Vec::new();
    for (i, path) in app.wallpapers.iter().enumerate() {
        if i % columns == 0 {
            lines.push(Line::default());
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let label = format!(" {:>3} {}", i + 1, truncate(name, 18));
        let style = if i == app.selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        if let Some(line) = lines.last_mut() {
            line.spans.push(Span::styled(format!("{label:<23}"), style));
        }
    }
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} wallpapers", app.wallpapers.len())),
        ),
        left[1],
    );
    frame.render_widget(
        Paragraph::new(app.message.as_str()).style(Style::default().fg(Color::Yellow)),
        left[2],
    );
    frame.render_widget(Clear, areas[1]);
    frame.render_widget(
        Paragraph::new("Kitty preview").block(
            Block::default().borders(Borders::ALL).title(
                selected
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("Preview"),
            ),
        ),
        areas[1],
    );
}

fn handle_key(app: &mut TuiApp, key: KeyEvent) -> Result<bool, Box<dyn std::error::Error>> {
    if app.editing {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => app.editing = false,
            KeyCode::Backspace => {
                app.search.pop();
                app.apply_filter();
            }
            KeyCode::Char(c) => {
                app.search.push(c);
                app.apply_filter();
            }
            _ => {}
        }
        return Ok(false);
    }
    match key {
        KeyEvent {
            code: KeyCode::Char('q'),
            ..
        }
        | KeyEvent {
            code: KeyCode::Esc, ..
        } => return Ok(true),
        KeyEvent {
            code: KeyCode::Char('/'),
            ..
        } => app.editing = true,
        KeyEvent {
            code: KeyCode::Up, ..
        } => app.move_selection(-4),
        KeyEvent {
            code: KeyCode::Down,
            ..
        } => app.move_selection(4),
        KeyEvent {
            code: KeyCode::Left,
            ..
        } => app.move_selection(-1),
        KeyEvent {
            code: KeyCode::Right,
            ..
        } => app.move_selection(1),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            if let Some(path) = app.selected_path().cloned() {
                match WallpaperService::new(Cache::load_default()).set_wallpaper(path) {
                    Ok(()) => app.message = "Wallpaper applied".into(),
                    Err(error) => app.message = error.to_string(),
                }
            }
        }
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => return Ok(true),
        _ => {}
    }
    Ok(false)
}

fn preview_rect(size: Size) -> Rect {
    Rect {
        x: size.width * 60 / 100 + 1,
        y: 1,
        width: size.width * 40 / 100 - 2,
        height: size.height.saturating_sub(2),
    }
}

fn draw_preview<W: Write>(stdout: &mut W, path: &PathBuf, area: Rect) -> io::Result<()> {
    if let Ok(output) = Command::new("chafa")
        .args([
            "--format=symbols",
            "--colors=full",
            "--polite=on",
            "--size",
            &format!(
                "{}x{}",
                area.width.saturating_sub(2),
                area.height.saturating_sub(2)
            ),
        ])
        .arg(path)
        .output()
    {
        if output.status.success() {
            execute!(stdout, MoveTo(area.x, area.y))?;
            stdout.write_all(&output.stdout)?;
            return stdout.flush();
        }
    }
    draw_kitty_preview(stdout, path, area)
}

fn draw_kitty_preview<W: Write>(stdout: &mut W, path: &PathBuf, area: Rect) -> io::Result<()> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(()),
    };
    let encoded = STANDARD.encode(bytes);
    write!(stdout, "\x1b_Ga=d,d=a,q=2\x1b\\")?;
    execute!(stdout, MoveTo(area.x, area.y))?;
    for (index, chunk) in encoded.as_bytes().chunks(4096).enumerate() {
        let more = if index + 1 < encoded.len().div_ceil(4096) {
            1
        } else {
            0
        };
        write!(
            stdout,
            "\x1b_Ga=T,f=100,c={},r={},m={};{}\x1b\\",
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
            more,
            String::from_utf8_lossy(chunk)
        )?;
    }
    stdout.flush()
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() > max {
        format!(
            "{}…",
            value
                .chars()
                .take(max.saturating_sub(1))
                .collect::<String>()
        )
    } else {
        value.into()
    }
}
