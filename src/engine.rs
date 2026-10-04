use crate::{
    app::App,
    camera::{Camera, CameraMovement},
    music::Music,
    object::Model,
    shader::Shader,
};
use glad_gl::gl::{COLOR_BUFFER_BIT, Clear, DEPTH_BUFFER_BIT, DEPTH_TEST, Enable};
use glam::{Mat4, Vec3};
use glfw::ffi::{
    GLFW_KEY_A, GLFW_KEY_D, GLFW_KEY_ESCAPE, GLFW_KEY_S, GLFW_KEY_W, GLFW_MOUSE_BUTTON_LEFT,
    GLFW_PRESS, GLFWwindow, glfwGetCursorPos, glfwGetKey, glfwGetMouseButton, glfwGetTime,
    glfwPollEvents, glfwSetWindowShouldClose, glfwSetWindowUserPointer, glfwSwapBuffers,
    glfwTerminate, glfwWindowShouldClose,
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
        let mut camera: Camera = Camera {
            position: Vec3::new(45.0, 25.0, 0.0),
            ..Default::default()
        };
        let mut last_x = WINDOW_WIDTH as f32 / 2.0;
        let mut last_y = WINDOW_HEIGHT as f32 / 2.0;
        let mut cursor_captured = false;
        let mut first_mouse = true;
        let mut last_frame = 0.0;
        let mut scroll_y = 0.0;

        let mut was_left_pressed = false;

        let mut app = App::new(WINDOW_WIDTH, WINDOW_HEIGHT, Self::WINDOW_DISPLAY_NAME);
        app.run()?;
        app.set_cursor_captured(cursor_captured);

        unsafe {
            Enable(DEPTH_TEST);
            glfwSetWindowUserPointer(
                app.window,
                &mut scroll_y as *mut f64 as *mut std::ffi::c_void,
            );
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

        shader.r#use();
        setup_light(&shader, &camera);

        unsafe {
            while glfwWindowShouldClose(app.window) == 0 {
                let current_frame = glfwGetTime() as f32;
                let delta_time = current_frame - last_frame;
                last_frame = current_frame;

                process_mouse_toggle(
                    app.window,
                    &app,
                    &mut cursor_captured,
                    &mut first_mouse,
                    &mut was_left_pressed,
                );

                process_input(app.window, &mut camera, delta_time);
                if cursor_captured {
                    process_mouse(
                        app.window,
                        &mut camera,
                        &mut first_mouse,
                        &mut last_x,
                        &mut last_y,
                    );
                };
                if scroll_y != 0.0 {
                    camera.process_mouse_scroll(scroll_y as f32);
                    scroll_y = 0.0;
                }

                rotate_view(&shader, &camera);
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
}

fn process_input(window: *mut GLFWwindow, camera: &mut Camera, delta_time: f32) {
    unsafe {
        if glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS {
            glfwSetWindowShouldClose(window, std::ffi::c_int::from(true));
        }
        if glfwGetKey(window, GLFW_KEY_W) == GLFW_PRESS {
            camera.process_keyboard(CameraMovement::Forward, delta_time);
        }
        if glfwGetKey(window, GLFW_KEY_S) == GLFW_PRESS {
            camera.process_keyboard(CameraMovement::Backward, delta_time);
        }
        if glfwGetKey(window, GLFW_KEY_A) == GLFW_PRESS {
            camera.process_keyboard(CameraMovement::Left, delta_time);
        }
        if glfwGetKey(window, GLFW_KEY_D) == GLFW_PRESS {
            camera.process_keyboard(CameraMovement::Right, delta_time);
        }
    }
}

fn process_mouse(
    window: *mut GLFWwindow,
    camera: &mut Camera,
    first_mouse: &mut bool,
    last_x: &mut f32,
    last_y: &mut f32,
) {
    let mut xpos = 0.0;
    let mut ypos = 0.0;

    unsafe { glfwGetCursorPos(window, &mut xpos, &mut ypos) };

    let xpos = xpos as f32;
    let ypos = ypos as f32;

    if *first_mouse {
        *last_x = xpos;
        *last_y = ypos;
        *first_mouse = false;
    }

    let xoffset = xpos - *last_x;
    let yoffset = *last_y - ypos; // reversed since y-coordinates go from bottom to top

    *last_x = xpos;
    *last_y = ypos;

    camera.process_mouse_movement(xoffset, yoffset, None);
}

fn process_mouse_toggle(
    window: *mut GLFWwindow,
    app: &App,
    mouse_captured: &mut bool,
    first_mouse: &mut bool,
    was_left_pressed: &mut bool,
) {
    let pressed = unsafe { glfwGetMouseButton(window, GLFW_MOUSE_BUTTON_LEFT) == GLFW_PRESS };

    if pressed && !*was_left_pressed {
        *mouse_captured = !*mouse_captured;
        app.set_cursor_captured(*mouse_captured);
        *first_mouse = true;
    }

    *was_left_pressed = pressed;
}

fn setup_light(light_shader: &Shader, camera: &Camera) {
    let point_light_positions = vec![
        Vec3::new(0.7, 0.2, 2.0),
        Vec3::new(2.3, -3.3, -4.0),
        Vec3::new(-4.0, 2.0, -12.0),
        Vec3::new(0.0, 0.0, -3.0),
    ];

    light_shader.set_vec3("viewPos", &camera.position);
    light_shader.set_float("material.shininess", 32.0);
    /*
       Here we set all the uniforms for the 5/6 types of lights we have. We have to set them manually and index
       the proper PointLight struct in the array to set each uniform variable. This can be done more code-friendly
       by defining light types as classes and set their values in there, or by using a more efficient uniform approach
       by using 'Uniform buffer objects', but that is something we'll discuss in the 'Advanced GLSL' tutorial.
    */
    // directional light
    light_shader.set_vec3(
        "dirLight.direction",
        &Vec3::new(-0.9486833, -0.31622776, 0.0),
    );
    light_shader.set_vec3("dirLight.ambient", &Vec3::new(0.05, 0.05, 0.05));
    light_shader.set_vec3("dirLight.diffuse", &Vec3::new(0.4, 0.4, 0.4));
    light_shader.set_vec3("dirLight.specular", &Vec3::new(0.5, 0.5, 0.5));
    // point light 1
    light_shader.set_vec3("pointLights[0].position", &point_light_positions[0]);
    light_shader.set_vec3("pointLights[0].ambient", &Vec3::new(0.05, 0.05, 0.05));
    light_shader.set_vec3("pointLights[0].diffuse", &Vec3::new(0.8, 0.8, 0.8));
    light_shader.set_vec3("pointLights[0].specular", &Vec3::new(1.0, 1.0, 1.0));
    light_shader.set_float("pointLights[0].constant", 1.0);
    light_shader.set_float("pointLights[0].linear", 0.09);
    light_shader.set_float("pointLights[0].quadratic", 0.032);
    // point light 2
    light_shader.set_vec3("pointLights[1].position", &point_light_positions[1]);
    light_shader.set_vec3("pointLights[1].ambient", &Vec3::new(0.05, 0.05, 0.05));
    light_shader.set_vec3("pointLights[1].diffuse", &Vec3::new(0.8, 0.8, 0.8));
    light_shader.set_vec3("pointLights[1].specular", &Vec3::new(1.0, 1.0, 1.0));
    light_shader.set_float("pointLights[1].constant", 1.0);
    light_shader.set_float("pointLights[1].linear", 0.09);
    light_shader.set_float("pointLights[1].quadratic", 0.032);
    // point light 3
    light_shader.set_vec3("pointLights[2].position", &point_light_positions[2]);
    light_shader.set_vec3("pointLights[2].ambient", &Vec3::new(0.05, 0.05, 0.05));
    light_shader.set_vec3("pointLights[2].diffuse", &Vec3::new(0.8, 0.8, 0.8));
    light_shader.set_vec3("pointLights[2].specular", &Vec3::new(1.0, 1.0, 1.0));
    light_shader.set_float("pointLights[2].constant", 1.0);
    light_shader.set_float("pointLights[2].linear", 0.09);
    light_shader.set_float("pointLights[2].quadratic", 0.032);
    // point light 4
    light_shader.set_vec3("pointLights[3].position", &point_light_positions[3]);
    light_shader.set_vec3("pointLights[3].ambient", &Vec3::new(0.05, 0.05, 0.05));
    light_shader.set_vec3("pointLights[3].diffuse", &Vec3::new(0.8, 0.8, 0.8));
    light_shader.set_vec3("pointLights[3].specular", &Vec3::new(1.0, 1.0, 1.0));
    light_shader.set_float("pointLights[3].constant", 1.0);
    light_shader.set_float("pointLights[3].linear", 0.09);
    light_shader.set_float("pointLights[3].quadratic", 0.032);
    // spotLight
    light_shader.set_vec3("spotLight.position", &camera.position);
    light_shader.set_vec3("spotLight.direction", &camera.front);
    light_shader.set_vec3("spotLight.ambient", &Vec3::new(0.0, 0.0, 0.0));
    light_shader.set_vec3("spotLight.diffuse", &Vec3::new(1.0, 1.0, 1.0));
    light_shader.set_vec3("spotLight.specular", &Vec3::new(1.0, 1.0, 1.0));
    light_shader.set_float("spotLight.constant", 1.0);
    light_shader.set_float("spotLight.linear", 0.09);
    light_shader.set_float("spotLight.quadratic", 0.032);
    light_shader.set_float("spotLight.cutOff", 12.5_f32.to_radians().cos());
    light_shader.set_float("spotLight.outerCutOff", 15f32.to_radians().cos());
}

fn rotate_view(shader: &Shader, camera: &Camera) {
    unsafe {
        let mut model = Mat4::from_rotation_x(-90.0_f32.to_radians());
        let time = glfwGetTime() as f32;
        model = model * Mat4::from_axis_angle(Vec3::Z, time * 3.0);

        let view = camera.get_view_matrix();
        let projection = glam::camera::rh::proj::opengl::perspective(
            camera.zoom.to_radians(),
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
