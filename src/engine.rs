use crate::{app::App, music::Music, object::Model, shader::Shader};
use glad_gl::gl::{COLOR_BUFFER_BIT, Clear, ClearColor, DEPTH_BUFFER_BIT, DEPTH_TEST, Enable};
use glam::{Mat4, Vec3};
use glfw::ffi::{
    GLFW_KEY_ESCAPE, GLFW_PRESS, GLFWwindow, glfwGetKey, glfwGetTime, glfwPollEvents,
    glfwSetWindowShouldClose, glfwSwapBuffers, glfwTerminate, glfwWindowShouldClose,
};

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const AUDIO_FILE: &str = "funkytown.mp3";
const PEAK_MODEL: &str = "models/12140_Skull_v3_L2.obj";

pub struct Engine {}

impl Engine {
    const VERTEX_SHADER_PATH: &str = "./shaders/vertex.glsl";
    const FRAGMENT_SHADER_PATH: &str = "./shaders/fragment.glsl";
    const WINDOW_DISPLAY_NAME: &str = "Cubey thing";

    pub fn run(&self) -> Result<(), &'static str> {
        let mut app = App::new(WINDOW_WIDTH, WINDOW_HEIGHT, Self::WINDOW_DISPLAY_NAME);
        app.run()?;

        unsafe {
            Enable(DEPTH_TEST);
        }

        let shader = Shader::new(Self::VERTEX_SHADER_PATH, Self::FRAGMENT_SHADER_PATH, None);
        let model = Model::new(PEAK_MODEL);

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
                rotate_view(&shader);
                ClearColor(1.0, 1.0, 1.0, 1.0);
                Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
                model.draw(&shader);
                glfwPollEvents();
                glfwSwapBuffers(app.window);
            }
        }

        drop(model);
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

fn rotate_view(shader: &Shader) {
    unsafe {
        let mut model = Mat4::from_rotation_x(-90.0_f32.to_radians());
        let time = glfwGetTime() as f32;
        model = model * Mat4::from_axis_angle(Vec3::Z, time * 3.0);

        let view = glam::camera::rh::view::look_at_mat4(
            Vec3::new(45.0, 25.0, 0.0), // Camera position
            Vec3::new(0.0, 10.0, 0.0),  // Look at the origin
            Vec3::new(0.0, 1.0, 0.0),   // Up vector (Y-axis)
        );
        let projection = glam::camera::rh::proj::opengl::perspective(
            45.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
            0.1,
            100.0,
        );

        shader.r#use();
        shader.set_mat4("model", &model);
        shader.set_mat4("view", &view);
        shader.set_mat4("projection", &projection);
    }
}
