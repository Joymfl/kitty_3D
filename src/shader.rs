use std::{
    env::{self},
    ffi::CString,
    fs, ptr,
};

use gl::types::{GLchar, GLint};
use walkdir::{DirEntry, WalkDir};

// shader file convention *.vert, *.frag etc.
pub struct ShaderProgram(u32);

impl ShaderProgram {
    // loads and compiles shaders from shaders/ dir
    pub fn load_and_compile() -> Result<Self, std::io::Error> {
        let mut current_dir = env::current_dir().unwrap();
        current_dir.push("shaders");
        let dirs: Vec<DirEntry> = WalkDir::new(current_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|f| f.file_type().is_file())
            .collect();
        let vertex_shader_file: Vec<&DirEntry> = dirs
            .iter()
            .filter(|&dir| {
                let split: Vec<&str> = dir.path().to_str().unwrap().split('.').collect();
                split[1] == "vert"
            })
            .collect();
        let fragment_shader_file: Vec<&DirEntry> = dirs
            .iter()
            .filter(|&dir| {
                let split: Vec<&str> = dir.path().to_str().unwrap().split('.').collect();
                split[1] == "frag"
            })
            .collect();

        // at least in this test, we're assuming the vertex & frag file exists.
        let vertex_shader_source = fs::read(vertex_shader_file.first().unwrap().path()).unwrap();
        let fragment_shader_source =
            fs::read(fragment_shader_file.first().unwrap().path()).unwrap();

        let shader_program = unsafe {
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let c_str_vert = CString::new(vertex_shader_source).unwrap();
            gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
            gl::CompileShader(vertex_shader);

            let mut success = gl::FALSE as GLint;
            let mut infolong = Vec::with_capacity(512);
            // infolong.set_len(512 - 1);
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                gl::GetShaderInfoLog(
                    vertex_shader,
                    512,
                    ptr::null_mut(),
                    infolong.as_mut_ptr() as *mut GLchar,
                );
                println!(
                    "ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}",
                    str::from_utf8(&infolong).unwrap()
                );
            }

            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let c_str_frag = CString::new(fragment_shader_source).unwrap();
            gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
            gl::CompileShader(fragment_shader);

            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                gl::GetShaderInfoLog(
                    fragment_shader,
                    512,
                    ptr::null_mut(),
                    infolong.as_mut_ptr() as *mut GLchar,
                );
                println!(
                    "ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}",
                    str::from_utf8(&infolong).unwrap()
                );
            }

            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);

            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                gl::GetProgramInfoLog(
                    shader_program,
                    512,
                    ptr::null_mut(),
                    infolong.as_mut_ptr() as *mut GLchar,
                );
            }
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            shader_program
        };

        Ok(Self(shader_program))
    }
}
