use crate::{app::App, figure::Figure, music::Music, shader::Shader};
use glad_gl::gl::GLuint;
use glfw::ffi::{
    GLFW_KEY_ESCAPE, GLFW_PRESS, GLFWwindow, glfwGetKey, glfwPollEvents, glfwSetWindowShouldClose,
    glfwSwapBuffers, glfwTerminate, glfwWindowShouldClose,
};

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const VERTICES: [f32; 27] = [
    -0.25, -0.5, 0.25, -0.25, -0.5, -0.25, 0.25, -0.5, -0.25, 0.25, -0.5, 0.25, -0.25, 0.0, 0.25,
    -0.25, 0.0, -0.25, 0.25, 0.0, -0.25, 0.25, 0.0, 0.25, 0.0, 0.4, 0.0,
];
const COLORS: [f32; 27] = [
    0.0745098, 0.0745098, 0.333333, 0.0745098, 0.0745098, 0.333333, 0.0745098, 0.0745098, 0.333333,
    0.0745098, 0.0745098, 0.333333, 0.0745098, 0.0745098, 0.333333, 0.0745098, 0.0745098, 0.333333,
    0.0745098, 0.0745098, 0.333333, 0.0745098, 0.0745098, 0.333333, 0.0745098, 0.0745098, 0.333333,
];
const INDICES: [GLuint; 32] = [
    0, 1, 1, 2, 2, 3, 3, 0, 0, 4, 4, 7, 7, 3, 7, 6, 6, 2, 6, 5, 5, 1, 5, 4, 8, 4, 8, 5, 8, 6, 8, 7,
];
const AUDIO_FILE: &str = "funkytown.mp3";

pub struct Engine {}

impl Engine {
    const VERTEX_SHADER_PATH: &str = "./shaders/vertex.glsl";
    const FRAGMENT_SHADER_PATH: &str = "./shaders/fragment.glsl";
    const WINDOW_DISPLAY_NAME: &str = "Cubey thing";

    pub fn run(&self) -> Result<(), &'static str> {
        let mut app = App::new(WINDOW_WIDTH, WINDOW_HEIGHT, Self::WINDOW_DISPLAY_NAME);
        app.run()?;

        let mut shader = Shader::new(Self::VERTEX_SHADER_PATH, Self::FRAGMENT_SHADER_PATH);
        let mut figure = Figure::new(
            VERTICES.as_ptr(),
            VERTICES.len(),
            INDICES.as_ptr(),
            INDICES.len(),
            COLORS.as_ptr(),
            COLORS.len(),
        );
        figure.set_shader(&mut shader);
        figure.setup_vertex_object();

        let music = match Music::new(AUDIO_FILE) {
            Ok(m) => m,
            Err(err) => {
                eprintln!("{:?}", err);
                return Err("Failed to load music");
            }
        };

        music.start();

        unsafe {
            while glfwWindowShouldClose(app.window) == 0 {
                self.process_input(app.window);
                figure.update(WINDOW_WIDTH, WINDOW_HEIGHT);
                figure.draw();
                glfwPollEvents();
                glfwSwapBuffers(app.window);
            }
        }

        drop(figure);
        drop(shader);
        drop(app);

        music.stop();
        unsafe {
            glfwTerminate();
        }

        Ok(())
    }

    pub fn process_input(&self, window: *mut GLFWwindow) {
        unsafe {
            if glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS {
                glfwSetWindowShouldClose(window, std::ffi::c_int::from(true));
            }
        }
    }
}
