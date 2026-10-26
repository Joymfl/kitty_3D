pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    textures: Vec<Texture>, // this is going to be unusued for no
    vao: u32,
}
impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            textures: vec![],
            vao: 0, //TODO: setup mesh and then return
        }
    }
}

struct Vertex {
    position: glm::Vec3,
    normal: glm::Vec3,
    tex_coords: glm::Vec2,
}

struct Texture {
    id: u32,
    tex_type: String,
}
