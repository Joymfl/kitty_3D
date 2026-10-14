use std::{
    collections::HashMap,
    env,
    error::Error,
    io::{self, Stderr},
};

extern crate khronos_egl as egl;

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

fn init_offscreen_buffer() -> Result<(), egl::Error> {
    let egl = egl::Instance::new(egl::Static);
    let display = unsafe { egl.get_display(egl::DEFAULT_DISPLAY) }.unwrap();
    let _ = egl.initialize(display).is_ok();

    let attributes = [
        egl::RED_SIZE,
        8,
        egl::GREEN_SIZE,
        8,
        egl::BLUE_SIZE,
        8,
        egl::NONE,
    ];
    egl.bind_api(egl::OPENGL_API)?;
    let config = egl
        .choose_first_config(display, &attributes)?
        .expect("Unable to find appropriate attrs");

    let context_attributes = [
        egl::CONTEXT_MAJOR_VERSION,
        4,
        egl::CONTEXT_MINOR_VERSION,
        0,
        egl::CONTEXT_OPENGL_PROFILE_MASK,
        egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
        egl::NONE,
    ];
    let context = egl.create_context(display, config, None, &context_attributes)?;
    let surface = unsafe { egl::Surface::from_ptr(egl::NO_SURFACE) };
    egl.make_current(display, Some(surface), Some(surface), Some(context))?;

    gl::load_with(|symbol| egl.get_proc_address(symbol).unwrap() as *mut std::ffi::c_void);

    unsafe {
        gl::ClearColor(1.0, 1.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Flush();
        gl::ReadBuffer(gl::BACK);
        let pixels: [u8; 1024] = std::array::from_fn(|i| 0);
        gl::ReadPixels(
            0,
            0,
            100,
            200,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            &pixels as *const u8 as *mut std::ffi::c_void,
        );
        for p in pixels.iter().enumerate() {
            println!("data {}: {}", p.0, p.1);
        }
    }

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

    match init_offscreen_buffer() {
        Ok(()) => {}
        Err(e) => println!("failed: {}", e),
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
