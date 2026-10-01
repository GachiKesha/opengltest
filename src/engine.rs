use crate::{app::App, figure::Figure, shader::Shader};
use glfw::ffi::{
    GLFW_KEY_ESCAPE, GLFW_PRESS, GLFWwindow, glfwGetKey, glfwPollEvents, glfwSetWindowShouldClose,
    glfwSwapBuffers, glfwTerminate, glfwWindowShouldClose,
};

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const VERTICES: [f32; 9] = [0.5, -0.5, 0.0, -0.5, -0.5, 0.0, 0.0, 0.5, 0.0];
const COLORS: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

pub struct Engine {}

impl Engine {
    const VERTEX_SHADER_PATH: &str = "./shaders/vertex.glsl";
    const FRAGMENT_SHADER_PATH: &str = "./shaders/fragment.glsl";
    const WINDOW_DISPLAY_NAME: &str = "LGBT triangle";

    pub fn run(&self) -> Result<(), &'static str> {
        let mut app = App::new(WINDOW_WIDTH, WINDOW_HEIGHT, Self::WINDOW_DISPLAY_NAME);
        app.run()?;

        let mut shader = Shader::new(Self::VERTEX_SHADER_PATH, Self::FRAGMENT_SHADER_PATH);
        let mut figure = Figure::new(
            VERTICES.as_ptr(),
            VERTICES.len(),
            COLORS.as_ptr(),
            COLORS.len(),
        );
        figure.set_shader(&mut shader);
        figure.setup_vertex_object();

        unsafe {
            while glfwWindowShouldClose(app.window) == 0 {
                self.process_input(app.window);
                figure.draw();
                glfwPollEvents();
                glfwSwapBuffers(app.window);
            }
        }

        drop(figure);
        drop(shader);
        drop(app);

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
