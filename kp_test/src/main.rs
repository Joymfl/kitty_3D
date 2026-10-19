use core::ffi;
use std::{
    collections::HashMap,
    env,
    error::Error,
    ffi::c_void,
    io::{self, Stderr},
    ptr::null,
};

extern crate khronos_egl as egl;

use libloading::Library;
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

use crate::device_test::{EGL_PLATFORM_WAYLAND_KHR, test_device};

pub(crate) mod device_test;
pub(crate) mod ui;

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
    let first_device = test_device().unwrap();

    run_app(&mut terminal, &mut app, first_device)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn init_offscreen_buffer(first_device: usize) -> Result<(), egl::Error> {
    let lib = unsafe { Library::new("libEGL.so") }.map_err(|_| egl::Error::NotInitialized)?;
    // handle nvidia registration via glvnd
    // TODO: Add support for other backends, or move to egl-wayland
    let egl = unsafe {
        egl::DynamicInstance::<egl::EGL1_5>::load_required_from(lib).expect("unable to load lib")
    };
    let display = unsafe {
        egl.get_platform_display(
            EGL_PLATFORM_WAYLAND_KHR,
            egl::DEFAULT_DISPLAY,
            &[egl::ATTRIB_NONE],
        )
    }?;
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
    // let surface = unsafe { egl::Surface::from_ptr(egl::NO_SURFACE) };
    let pbuffer_attrib_list = [egl::WIDTH, 512, egl::HEIGHT, 512, egl::NONE];
    let pbuffer = egl.create_pbuffer_surface(display, config, &pbuffer_attrib_list)?;
    egl.make_current(display, Some(pbuffer), Some(pbuffer), Some(context))?;

    gl::load_with(|symbol| egl.get_proc_address(symbol).unwrap() as *mut std::ffi::c_void);

    let (mut fbo, mut tex) = (0, 0);
    unsafe {
        gl::GenFramebuffers(1, &mut fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            100,
            200,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            null(),
        );

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            tex,
            0,
        );
    }

    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
        gl::ClearColor(1.0, 1.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Flush();
        gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        // width * height * bytes required for color format
        let pixels: [u8; 80_000] = std::array::from_fn(|_| 0);
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
            if p.0 > 10_000 {
                break;
            }
            // println!("data {}: {}", p.0, p.1);
        }
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    first_device: usize,
) -> io::Result<bool>
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

    match init_offscreen_buffer(first_device) {
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
