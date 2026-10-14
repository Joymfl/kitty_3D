use std::path::Path;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::App;

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
pub fn ui(frame: &mut Frame, app: &mut App) {
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

    // if let Some(path) = model_path.take() {
    //     load_model_info(path.to_str().unwrap()).unwrap();
    // }
}
