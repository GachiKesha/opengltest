use glad_gl::gl::{
    ARRAY_BUFFER, BindBuffer, BindVertexArray, BufferData, COLOR_BUFFER_BIT, Clear, ClearColor,
    DeleteBuffers, DeleteVertexArrays, DrawElements, ELEMENT_ARRAY_BUFFER, EnableVertexAttribArray,
    FALSE, FLOAT, FRONT_AND_BACK, GLint, GLsizei, GLsizeiptr, GLuint, GenBuffers, GenVertexArrays,
    GetUniformLocation, LINE, LINES, LineWidth, PolygonMode, STATIC_DRAW, UNSIGNED_INT,
    UniformMatrix4fv, VertexAttribPointer,
};
use glam::{Mat4, Vec3};
use glfw::ffi::glfwGetTime;

use super::shader::Shader;

use std::{
    ffi::{CString, c_void},
    ptr::{null, null_mut},
};

pub struct Figure {
    pub vao: GLuint,
    pub vbo: [GLuint; 2],
    pub ebo: GLuint,
    vertices: *const f32,
    colors: *const f32,
    indices: *const GLuint,
    vertices_count: usize,
    colors_count: usize,
    indices_count: usize,
    shader_ptr: *mut Shader,
}

impl Figure {
    pub fn new(
        vertices: *const f32,
        vertices_count: usize,
        indices: *const GLuint,
        indices_count: usize,
        colors: *const f32,
        colors_count: usize,
    ) -> Self {
        Self {
            vao: 0,
            vbo: [0, 0],
            ebo: 0,
            vertices,
            colors,
            indices,
            vertices_count,
            colors_count,
            indices_count,
            shader_ptr: null_mut(),
        }
    }

    pub fn setup_vertex_object(&mut self) {
        unsafe {
            PolygonMode(FRONT_AND_BACK, LINE);
            GenBuffers(2, self.vbo.as_mut_ptr());
            GenBuffers(1, &mut self.ebo);
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

            BindBuffer(ELEMENT_ARRAY_BUFFER, self.ebo);
            BufferData(
                ELEMENT_ARRAY_BUFFER,
                (std::mem::size_of::<GLuint>() * self.indices_count) as GLsizeiptr,
                self.indices.cast(),
                STATIC_DRAW,
            );

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
                3 * std::mem::size_of::<f32>() as GLsizei,
                null::<c_void>(),
            );
            EnableVertexAttribArray(1);
        }
    }

    pub fn draw(&mut self, scr_width: i32, scr_height: i32) {
        unsafe {
            ClearColor(1.0, 1.0, 1.0, 1.0);
            LineWidth(3.0);
            Clear(COLOR_BUFFER_BIT);
            (*self.shader_ptr).r#use();

            // create transformations
            let mut model = Mat4::IDENTITY; // make sure to initialize matrix to identity matrix first
            let view = glam::camera::rh::view::look_at_mat4(
                Vec3::new(0.5, 0.5, 2.0), // Camera position
                Vec3::new(0.0, 0.0, 0.0), // Look at the origin
                Vec3::new(0.0, 1.0, 0.0), // Up vector (Y-axis)
            );
            let projection = glam::camera::rh::proj::opengl::perspective(
                45.0_f32.to_radians(),
                scr_width as f32 / scr_height as f32,
                0.1,
                100.0,
            );
            model = model * Mat4::from_axis_angle(Vec3::Y, glfwGetTime() as f32 * 3.0);
            // retrieve the matrix uniform locations
            let model_name = CString::new("model").unwrap();
            let view_name = CString::new("view").unwrap();
            let model_loc =
                GetUniformLocation((*self.shader_ptr).shader_program, model_name.as_ptr());
            let view_loc =
                GetUniformLocation((*self.shader_ptr).shader_program, view_name.as_ptr());
            // pass them to the shaders (3 different ways)
            UniformMatrix4fv(model_loc, 1, FALSE, model.to_cols_array().as_ptr());
            UniformMatrix4fv(view_loc, 1, FALSE, &view.x_axis.x);
            // note: currently we set the projection matrix each frame, but since the projection matrix rarely changes it's often best practice to set it outside the main loop only once.
            (*self.shader_ptr).set_mat4("projection", &projection);

            BindVertexArray(self.vao);
            DrawElements(
                LINES,
                self.indices_count as GLsizei,
                UNSIGNED_INT,
                std::ptr::null(),
            );
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
            DeleteBuffers(1, &self.ebo);
        }
    }
}
