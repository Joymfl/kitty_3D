use std::{
    collections::HashMap,
    env,
    error::Error,
    io::{self, Stderr},
};

use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use walkdir::{DirEntry, WalkDir};

pub mod ui;

// TODO: make it more generic, this constrains it to crossterm backend and stderr writer
fn setup() -> Result<Terminal<CrosstermBackend<Stderr>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();

    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub struct App {
    //TODO: should be owned by prompt system
    pub model_list: HashMap<String, DirEntry>, //filepaths
    pub current_row: usize,
    pub run: bool,
}

impl App {
    fn new() -> Self {
        Self {
            model_list: HashMap::new(),
            current_row: 0,
            run: true,
        }
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup()?;

    let mut app = App::new();

    run_app(&mut terminal, &mut app)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool>
where
    io::Error: From<B::Error>,
{
    let mut current_dir = env::current_dir().unwrap();
    current_dir.push("models");
    let dirs: Vec<DirEntry> = WalkDir::new(current_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.file_type().is_file())
        .collect();

    for dir in dirs {
        app.model_list
            .insert(dir.file_name().to_str().unwrap().to_string(), dir);
    }

    // render loop
    loop {
        terminal.draw(|f| ui::ui(f, app))?;
        handle_event(app).map_err(|err| io::Error::other(err.to_string()))?;
        if !app.run {
            return Ok(true);
        }
    }
}

fn handle_event(app: &mut App) -> Result<(), Box<dyn Error>> {
    if let Event::Key(key) = event::read()? {
        if key.kind == event::KeyEventKind::Release {
            return Ok(());
        }
        match key.code {
            KeyCode::Char('q') => {
                app.run = false;
                return Ok(());
            }
            KeyCode::Char('j') if app.current_row != app.model_list.keys().len() - 1 => {
                app.current_row += 1;
            }
            KeyCode::Char('k') if app.current_row != 0 => {
                app.current_row -= 1;
            }
            _ => {}
        }
    }
    Ok(())
}
