use std::{
    io::{self},
    num::NonZero,
    path::Path,
    rc::Rc,
};

use crossterm::{cursor::MoveTo, execute, terminal::WindowSize};
use gl::types::GLuint;
use kittage::{
    ImageDimensions, InputReader,
    action::Action,
    display::{DisplayConfig, DisplayLocation},
    image::Image,
    medium::SharedMemObject,
    tmux::TmuxWriter,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};
use russimp_ng::{
    RussimpError,
    mesh::Mesh as AssimpMesh,
    scene::{PostProcess, Scene},
};

use crate::App;

struct KpErr(String);

impl InputReader for KpErr {
    type Error = io::Error;

    fn read_esc_delimited_str(&mut self, buf: &mut String) -> Result<(), Self::Error> {
        self.0 = buf.to_owned();
        Ok(())
    }
}

//TODO: just switch to ratatui backend trait
pub fn get_terminal_in_cells() -> io::Result<WindowSize> {
    //NOTE: This works with tmux  and resizing, but resize needs a callback hook, but as noted in
    //docs does not work in windows and might not be supported in all platforms
    let win_size = crossterm::terminal::window_size()?;
    Ok(win_size)
}

// shared between the render loop (for sizing the fbo texture) and ui (for drawing)
pub fn layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Min(1)])
        .split(area)
}

// region in cells -> (width, height) in px
pub fn region_in_px(win_size: &WindowSize, region: Rect) -> (u32, u32) {
    // singular cell
    let cell_x = win_size.width / win_size.columns;
    let cell_y = win_size.height / win_size.rows;
    (
        (cell_x * region.width) as u32,
        (cell_y * region.height) as u32,
    )
}

// Should be 1 instance for the whole app currently
pub struct DrawContext {
    fbo: GLuint,
    width: u32,  // px
    height: u32, // px
    frame: u64,
    offset_x: u16, // in cells
}

impl DrawContext {
    pub fn new(fbo: GLuint, width: u32, height: u32, frame: u64, offset_x: u16) -> Self {
        Self {
            fbo,
            width,
            height,
            frame,
            offset_x,
        }
    }
    pub fn handle_resize(&mut self, width: u32, height: u32, offset_x: u16) {
        self.width = width;
        self.height = height;
        self.offset_x = offset_x;
    }
}

// NOTE: should always be the last call in the overall render loop
// width, height = in px, must match the fbo texture allocation
fn gl_draw_loop(draw_context: &DrawContext, mesh: AssimpMesh) {
    execute!(io::stdout(), MoveTo(draw_context.offset_x, 0)).unwrap();

    // width * height * bytes required for color format
    let area = (draw_context.width) as usize * (draw_context.height) as usize;
    let mut pixels = vec![0u8; area * 4];
    let name = format!("kp_test-{}", draw_context.frame);
    let mut shm = SharedMemObject::create_new(&name, area * 4).unwrap();
    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, draw_context.fbo);
        gl::ClearColor(1.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Flush();
        gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        gl::ReadPixels(
            0,
            0,
            draw_context.width as i32,
            draw_context.height as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr().cast(),
        );
    }
    shm.copy_in_buf(&pixels).unwrap();
    let image = Image {
        num_or_id: kittage::NumberOrId::Number(NonZero::new(1).unwrap()),
        format: kittage::PixelFormat::Rgba32(
            ImageDimensions {
                width: draw_context.width,
                height: draw_context.height,
            },
            None,
        ),
        medium: kittage::medium::Medium::SharedMemObject(shm),
    };
    let display_config = DisplayConfig {
        location: DisplayLocation {
            x: 0,
            y: 0,
            width: draw_context.width,
            height: draw_context.height,
            x_offset: 0,
            y_offset: 0,
            columns: 0,
            rows: 0,
            z_index: 5,
            horizontal_offset: 0,
            vertical_offset: 0,
        },
        cursor_movement: kittage::display::CursorMovementPolicy::DontMove,
        create_virtual_placement: false,
        parent_id: None,
        parent_placement: None,
    };
    let action = Action::TransmitAndDisplay {
        image,
        config: display_config,
        placement_id: None,
    };
    //TODO: does this actually work if the backend is not tmux? Check
    let out = TmuxWriter::new(io::stdout());
    let _ = action.write_transmit_to(out, kittage::Verbosity::ErrorsOnly);
    //NOTE:  race condition here. If you don't forget action, shm gets dropped before the tty can actually
    // read it. Another way to solve this would be to use `action.execute()` but, that blocks until
    // tty sends a response. A tradeoff with `mem::forget` is that we have no idea currently with
    // what the tty responds with, so errors go unhandled
    std::mem::forget(action)
    // action.execute(out, KpErr("".to_string()));
}

//TODO: make async, this is blocking the render
fn load_model_info(file_path: Option<&Path>) -> Result<Vec<AssimpMesh>, RussimpError> {
    let post_process_flags = vec![
        PostProcess::CalculateTangentSpace,
        PostProcess::Triangulate,
        PostProcess::JoinIdenticalVertices,
        PostProcess::SortByPrimitiveType,
        PostProcess::GenerateNormals,
        PostProcess::GenerateUVCoords,
    ];
    if let Some(path) = file_path {
        // TODO: Extract mesh ddata and pass to egl
        let scene: Scene = Scene::from_file(path.to_str().unwrap(), post_process_flags)?;
        Ok(scene.meshes)
    } else {
        Err(RussimpError::Import("File not found".to_string()))
    }
}

// For testing purposes, only 1 context during runtime.
pub fn ui(frame: &mut Frame, app: &mut App, draw_context: &mut DrawContext) {
    let chunks = layout(frame.area());

    let list_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let active_span_style = Style::default().fg(Color::White).bg(Color::LightYellow);
    let normal_span_style = Style::default().fg(Color::Yellow);
    let mut list_items = Vec::<ListItem>::new();
    let mut model_path: Option<&Path> = None;
    // TODO: Horrible hack of indices, this is unstable. Right now it works, since the hashmap
    // doesn't move, but this is unstable.
    for key in app.model_list.keys().enumerate() {
        if key.0 == app.current_row {
            model_path = Some(app.model_list.get(key.1).unwrap().path());
        }
        list_items.push(ListItem::new(Line::from(Span::styled(
            key.1,
            if key.0 == app.current_row {
                active_span_style
            } else {
                normal_span_style
            },
        ))));
    }
    let list = List::new(list_items).block(list_block);
    frame.render_widget(list, chunks[0]);

    let mut meshes = load_model_info(model_path).unwrap();
    gl_draw_loop(draw_context, std::mem::take(meshes.first_mut().unwrap()));
    app.frame += 1;
}
