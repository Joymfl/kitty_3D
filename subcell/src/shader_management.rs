use std::path::Path;

struct Shader {
    s_type: ShaderType,
    path: Path,
}

enum ShaderType {
    Vertex,
    Fragment,
}
