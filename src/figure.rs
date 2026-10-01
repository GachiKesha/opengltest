use glad_gl::gl::{
    ARRAY_BUFFER, BindBuffer, BindVertexArray, BufferData, COLOR_BUFFER_BIT, Clear, ClearColor,
    DeleteBuffers, DeleteVertexArrays, DrawArrays, EnableVertexAttribArray, FALSE, FLOAT, GLint,
    GLsizeiptr, GLuint, GenBuffers, GenVertexArrays, STATIC_DRAW, TRIANGLES, VertexAttribPointer,
};

use super::shader::Shader;

use std::{
    ffi::c_void,
    ptr::{null, null_mut},
};

pub struct Figure {
    pub vao: GLuint,
    pub vbo: [GLuint; 2],
    vertices: *const f32,
    colors: *const f32,
    vertices_count: usize,
    colors_count: usize,
    shader_ptr: *mut Shader,
}

impl Figure {
    pub fn new(
        vertices: *const f32,
        vertices_count: usize,
        colors: *const f32,
        colors_count: usize,
    ) -> Self {
        Self {
            vao: 0,
            vbo: [0, 0],
            vertices,
            colors,
            vertices_count,
            colors_count,
            shader_ptr: null_mut(),
        }
    }

    pub fn setup_vertex_object(&mut self) {
        unsafe {
            GenBuffers(2, self.vbo.as_mut_ptr());
            GenVertexArrays(1, &mut self.vao);
            BindVertexArray(self.vao);

            BindBuffer(ARRAY_BUFFER, self.vbo[0]);
            BufferData(
                ARRAY_BUFFER,
                (std::mem::size_of::<f32>() * self.vertices_count) as GLsizeiptr,
                self.vertices.cast(),
                STATIC_DRAW,
            );
            VertexAttribPointer(
                0,
                3,
                FLOAT,
                FALSE,
                3 * std::mem::size_of::<f32>() as GLint,
                null::<c_void>(),
            );
            EnableVertexAttribArray(0);

            BindBuffer(ARRAY_BUFFER, self.vbo[1]);
            BufferData(
                ARRAY_BUFFER,
                (std::mem::size_of::<f32>() * self.colors_count) as GLsizeiptr,
                self.colors.cast(),
                STATIC_DRAW,
            );
            VertexAttribPointer(
                1,
                3,
                FLOAT,
                FALSE,
                3 * std::mem::size_of::<f32>() as GLint,
                null::<c_void>(),
            );
            EnableVertexAttribArray(1);
        }
    }

    pub fn draw(&mut self) {
        unsafe {
            ClearColor(0.2, 0.3, 0.3, 1.0);
            Clear(COLOR_BUFFER_BIT);
            (*self.shader_ptr).r#use();
            BindVertexArray(self.vao);
            DrawArrays(TRIANGLES, 0, 3);
        }
    }

    pub fn set_shader(&mut self, shader: *mut Shader) {
        self.shader_ptr = shader;
    }
}

impl Drop for Figure {
    fn drop(&mut self) {
        unsafe {
            DeleteVertexArrays(1, &self.vao);
            DeleteBuffers(2, self.vbo.as_ptr());
        }
    }
}
