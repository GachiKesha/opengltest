use crate::shader::Shader;
use glad_gl::gl::{
    ARRAY_BUFFER, ActiveTexture, BindBuffer, BindTexture, BindVertexArray, BufferData,
    DrawElements, ELEMENT_ARRAY_BUFFER, EnableVertexAttribArray, FALSE, FLOAT, GLsizei, GLsizeiptr,
    GLuint, GenBuffers, GenVertexArrays, LINES, LineWidth, STATIC_DRAW, TEXTURE_2D, TEXTURE0,
    UNSIGNED_INT, VertexAttribPointer,
};
use glam::{Vec2, Vec3};
use std::ffi::c_void;
use std::mem::{offset_of, size_of};

pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coords: Vec2,
}

#[derive(Clone)]
pub struct Texture {
    pub id: GLuint,
    pub r#type: String,
    pub path: String,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<GLuint>,
    pub textures: Vec<Texture>,
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<GLuint>, textures: Vec<Texture>) -> Self {
        let mut mesh = Self {
            vertices,
            indices,
            textures,
            vao: 0,
            vbo: 0,
            ebo: 0,
        };
        mesh.setup_mesh();
        mesh
    }

    fn setup_mesh(&mut self) {
        unsafe {
            GenBuffers(1, &mut self.vbo);
            GenBuffers(1, &mut self.ebo);
            GenVertexArrays(1, &mut self.vao);
            BindVertexArray(self.vao);

            BindBuffer(ARRAY_BUFFER, self.vbo);
            BufferData(
                ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<Vertex>()) as GLsizeiptr,
                self.vertices.as_ptr() as *const c_void,
                STATIC_DRAW,
            );

            BindBuffer(ELEMENT_ARRAY_BUFFER, self.ebo);
            BufferData(
                ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * std::mem::size_of::<GLuint>()) as GLsizeiptr,
                self.indices.as_ptr() as *const c_void,
                STATIC_DRAW,
            );

            // vertex positions
            EnableVertexAttribArray(0);
            VertexAttribPointer(
                0,
                3,
                FLOAT,
                FALSE,
                size_of::<Vertex>() as GLsizei,
                offset_of!(Vertex, position) as *const c_void,
            );

            // vertex normals
            EnableVertexAttribArray(1);
            VertexAttribPointer(
                1,
                3,
                FLOAT,
                FALSE,
                size_of::<Vertex>() as GLsizei,
                offset_of!(Vertex, normal) as *const c_void,
            );

            // vertex texture coords
            EnableVertexAttribArray(2);
            VertexAttribPointer(
                2,
                2,
                FLOAT,
                FALSE,
                size_of::<Vertex>() as GLsizei,
                offset_of!(Vertex, tex_coords) as *const c_void,
            );

            BindVertexArray(0);
        }
    }

    pub fn draw(&self, shader: &Shader) {
        unsafe {
            let mut diffuse_nr = 1u32;
            let mut specular_nr = 1u32;

            for i in 0..self.textures.len() {
                ActiveTexture(TEXTURE0 + i as u32);
                let mut number = String::new();
                let name = &self.textures[i].r#type;
                if name == "texture__diffuse" {
                    diffuse_nr += 1;
                    number = diffuse_nr.to_string();
                } else if name == "texture_specular" {
                    specular_nr += 1;
                    number = specular_nr.to_string();
                }

                shader.set_int(&format!("material.{}{}", name, number), i as i32);
                BindTexture(TEXTURE_2D, self.textures[i].id);
            }

            ActiveTexture(TEXTURE0);

            // draw mesh
            LineWidth(3.0);
            BindVertexArray(self.vao);
            DrawElements(
                LINES,
                self.indices.len() as GLsizei,
                UNSIGNED_INT,
                std::ptr::null(),
            );
            BindVertexArray(0);
        }
    }
}
