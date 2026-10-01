use glad_gl::gl::{
    AttachShader, COMPILE_STATUS, CompileShader, CreateProgram, CreateShader, DeleteProgram,
    DeleteShader, FRAGMENT_SHADER, GLfloat, GLint, GLuint, GetProgramInfoLog, GetProgramiv,
    GetShaderInfoLog, GetShaderiv, GetUniformLocation, LINK_STATUS, LinkProgram, ShaderSource,
    Uniform1f, Uniform1i, UseProgram, VERTEX_SHADER,
};
use std::ffi::CString;
pub struct Shader {
    pub shader_program: GLuint,
}

impl Shader {
    pub fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> Self {
        let vertex_shader_sourcestr = Shader::read_shader_file(vertex_shader_path);
        let fragment_shader_sourcestr = Shader::read_shader_file(fragment_shader_path);

        let vertex_shader_source = CString::new(vertex_shader_sourcestr).unwrap();
        let fragment_shader_source = CString::new(fragment_shader_sourcestr).unwrap();

        unsafe {
            let vertex_shader = CreateShader(VERTEX_SHADER);
            ShaderSource(
                vertex_shader,
                1,
                &vertex_shader_source.as_ptr(),
                std::ptr::null(),
            );
            CompileShader(vertex_shader);
            Shader::check_compile_errors(vertex_shader, "VERTEX");

            let fragment_shader: GLuint;
            fragment_shader = CreateShader(FRAGMENT_SHADER);
            ShaderSource(
                fragment_shader,
                1,
                &fragment_shader_source.as_ptr(),
                std::ptr::null(),
            );
            CompileShader(fragment_shader);
            Shader::check_compile_errors(fragment_shader, "FRAGMENT");

            let shader_program = CreateProgram();
            AttachShader(shader_program, vertex_shader);
            AttachShader(shader_program, fragment_shader);
            LinkProgram(shader_program);
            Shader::check_compile_errors(shader_program, "PROGRAM");

            DeleteShader(vertex_shader);
            DeleteShader(fragment_shader);

            Self { shader_program }
        }
    }

    pub fn r#use(&self) {
        unsafe {
            UseProgram(self.shader_program);
        }
    }

    pub fn set_bool(&self, name: &str, value: bool) {
        let name = CString::new(name).unwrap();
        unsafe {
            Uniform1i(
                GetUniformLocation(self.shader_program, name.as_ptr()),
                value as GLint,
            );
        }
    }

    pub fn set_int(&self, name: &str, value: GLint) {
        let name = CString::new(name).unwrap();
        unsafe {
            Uniform1i(
                GetUniformLocation(self.shader_program, name.as_ptr()),
                value,
            );
        }
    }

    pub fn set_float(&self, name: &str, value: GLfloat) {
        let name = CString::new(name).unwrap();
        unsafe {
            Uniform1f(
                GetUniformLocation(self.shader_program, name.as_ptr()),
                value,
            );
        }
    }

    fn read_shader_file(filename: &str) -> String {
        match std::fs::read_to_string(filename) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Failed to open shader file {}: {}", filename, e);
                String::new()
            }
        }
    }

    fn check_compile_errors(shader: GLuint, r#type: &str) {
        unsafe {
            let mut success: GLint = 0;
            let mut info_log = vec![0i8; 1024];
            if r#type == "PROGRAM" {
                GetProgramiv(shader, LINK_STATUS, &mut success);
                if success == 0 {
                    GetProgramInfoLog(shader, 1024, std::ptr::null_mut(), info_log.as_mut_ptr());
                    println!(
                        "ERROR::SHADER::{}::LINKING_FAILED\n{:?}\n -- --------------------------------------------------- -- ",
                        r#type, info_log
                    );
                }
            } else {
                GetShaderiv(shader, COMPILE_STATUS, &mut success);
                if success == 0 {
                    GetShaderInfoLog(shader, 1024, std::ptr::null_mut(), info_log.as_mut_ptr());
                    println!(
                        "ERROR::SHADER::{}::COMPILATION_FAILED\n{:?}\n -- --------------------------------------------------- -- ",
                        r#type, info_log
                    );
                }
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            DeleteProgram(self.shader_program);
        }
    }
}
