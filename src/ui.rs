use std::{
    borrow::Cow,
    io::{self},
    num::NonZero,
};

use crossterm::cursor::MoveTo;
use gl::types::GLuint;
use kittage::{
    ImageDimensions, InputReader,
    action::Action,
    display::{DisplayConfig, DisplayLocation},
    image::Image,
    medium::ChunkSize,
    tmux::TmuxWriter,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
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

// NOTE: should always be the last call in the overall render loop
fn gl_draw_loop(fbo: GLuint) {
    // width * height * bytes required for color format
    let mut pixels = vec![0u8; 100 * 200 * 4];
    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
        gl::ClearColor(1.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Flush();
        gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        gl::ReadPixels(
            0,
            0,
            100,
            200,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr().cast(),
        );
    }
    let image = Image {
        num_or_id: kittage::NumberOrId::Number(NonZero::new(1).unwrap()),
        format: kittage::PixelFormat::Rgba32(
            ImageDimensions {
                width: 100,
                height: 200,
            },
            None,
        ),
        medium: kittage::medium::Medium::Direct {
            chunk_size: Some(ChunkSize::default()),
            data: Cow::from(&pixels),
        },
    };
    let display_config = DisplayConfig {
        location: DisplayLocation {
            x: 0,
            y: 0,
            width: 100,
            height: 200,
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
    let out = TmuxWriter::new(io::stdout());
    let _ = action.write_transmit_to(out, kittage::Verbosity::ErrorsOnly);
}
// fn load_model_info(file_path: &str) -> Result<(), RussimpError> {
//     let post_process_flags = vec![
//         PostProcess::CalculateTangentSpace,
//         PostProcess::Triangulate,
//         PostProcess::JoinIdenticalVertices,
//         PostProcess::SortByPrimitiveType,
//         PostProcess::GenerateNormals,
//         PostProcess::GenerateUVCoords,
//     ];
//     // TODO: Extract mesh ddata and pass to egl
//     // let scene: Scene = Scene::from_file(file_path, post_process_flags)?;
//     // println!("scnee loaded succesfully, from: {}", file_path);
//     // println!("Meshes: {}", scene.meshes.len());
//     Ok(())
// }
//
// For testing purposes, only 1 context during runtime.
pub fn ui(frame: &mut Frame, app: &mut App, framebuffer_id: GLuint, texture_id: GLuint) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Min(1)])
        .split(frame.area());

    let list_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let active_span_style = Style::default().fg(Color::White).bg(Color::LightYellow);
    let normal_span_style = Style::default().fg(Color::Yellow);
    let mut list_items = Vec::<ListItem>::new();
    // let mut model_path: Option<&Path> = None;
    // TODO: Horrible hack of indices, this is unstable. Right now it works, since the hashmap
    // doesn't move, but this is unstable.
    for key in app.model_list.keys().enumerate() {
        if key.0 == app.current_row {
            // model_path = Some(app.model_list.get(key.1).unwrap().path());
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

    // if let Some(path) = model_path.take() {
    //     load_model_info(path.to_str().unwrap()).unwrap();
    // }
    gl_draw_loop(framebuffer_id);
}
