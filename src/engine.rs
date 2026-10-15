use crate::{
    app::App,
    camera::{Camera, CameraMovement},
    music::Music,
    object::Model,
    shader::Shader,
};
use glad_gl::gl::{COLOR_BUFFER_BIT, Clear, DEPTH_BUFFER_BIT, DEPTH_TEST, Enable, Viewport};
use glam::{Mat4, Vec2, Vec3};
use sdl3::sys::{
    events::{
        SDL_EVENT_KEY_DOWN, SDL_EVENT_MOUSE_BUTTON_DOWN, SDL_EVENT_MOUSE_MOTION,
        SDL_EVENT_MOUSE_WHEEL, SDL_EVENT_QUIT, SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED,
        SDL_EVENT_WINDOW_RESIZED, SDL_Event, SDL_EventType, SDL_PollEvent,
    },
    init::SDL_Quit,
    keycode::{SDLK_A, SDLK_D, SDLK_ESCAPE, SDLK_S, SDLK_W},
    mouse::SDL_BUTTON_LEFT,
    timer::SDL_GetTicks,
    video::SDL_GL_SwapWindow,
};

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const AUDIO_FILE: &str = "bad-to-the-bone.mp3";
const PEAK_MODEL: &str = "models/12140_Skull_v3_L2.obj";

pub struct Engine {}

impl Engine {
    const VERTEX_SHADER_PATH: &str = "./shaders/vertex.glsl";
    const FRAGMENT_SHADER_PATH: &str = "./shaders/fragment.glsl";
    const WINDOW_DISPLAY_NAME: &str = "Cubey thing";

    pub fn run(&self) -> Result<(), String> {
        let mut camera: Camera = Camera {
            position: Vec3::new(45.0, 25.0, 0.0),
            ..Default::default()
        };
        let mut last_x = WINDOW_WIDTH as f32 / 2.0;
        let mut last_y = WINDOW_HEIGHT as f32 / 2.0;
        let mut cursor_captured = false;
        let mut first_mouse = true;
        let mut last_frame = 0.0;
        let mut was_facing_camera = false;

        let mut app = App::new(WINDOW_WIDTH, WINDOW_HEIGHT, Self::WINDOW_DISPLAY_NAME);
        app.run()?;
        app.set_cursor_captured(cursor_captured);

        unsafe {
            Enable(DEPTH_TEST);
        }

        let shader = Shader::new(Self::VERTEX_SHADER_PATH, Self::FRAGMENT_SHADER_PATH, None);
        let model = Model::new(PEAK_MODEL);

        let music = match Music::new(AUDIO_FILE) {
            Ok(m) => m,
            Err(err) => {
                eprintln!("{:?}", err);
                return Err("Failed to load music".to_string());
            }
        };

        // music.start();

        shader.r#use();
        setup_light(&shader, &camera);

        let mut event = SDL_Event::default();
        let mut running = true;

        unsafe {
            while running {
                let current_frame = SDL_GetTicks() as f32 / 1000.0;
                let delta_time = current_frame - last_frame;
                last_frame = current_frame;

                while SDL_PollEvent(&mut event) {
                    match SDL_EventType(event.r#type) {
                        SDL_EVENT_QUIT => running = false,
                        SDL_EVENT_WINDOW_RESIZED => {
                            Viewport(0, 0, event.window.data1, event.window.data2)
                        }
                        SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED => {
                            let width = event.window.data1;
                            let height = event.window.data2;
                            Viewport(0, 0, width, height);
                        }
                        SDL_EVENT_KEY_DOWN => match event.key.key {
                            SDLK_ESCAPE => running = false,
                            SDLK_W => camera.process_keyboard(CameraMovement::Forward, delta_time),
                            SDLK_S => camera.process_keyboard(CameraMovement::Backward, delta_time),
                            SDLK_A => camera.process_keyboard(CameraMovement::Left, delta_time),
                            SDLK_D => camera.process_keyboard(CameraMovement::Right, delta_time),
                            _ => {}
                        },
                        SDL_EVENT_MOUSE_BUTTON_DOWN => {
                            if event.button.button == SDL_BUTTON_LEFT as u8 {
                                cursor_captured = !cursor_captured;
                                app.set_cursor_captured(cursor_captured);
                                first_mouse = true;
                            }
                        }
                        SDL_EVENT_MOUSE_WHEEL => camera.process_mouse_scroll(event.wheel.y),
                        SDL_EVENT_MOUSE_MOTION => {
                            if cursor_captured {
                                let motion = event.motion;

                                let xpos = motion.x;
                                let ypos = motion.y;

                                if first_mouse {
                                    last_x = xpos;
                                    last_y = ypos;
                                    first_mouse = false;
                                }

                                let xoffset = xpos - last_x;
                                let yoffset = last_y - ypos;

                                last_x = xpos;
                                last_y = ypos;

                                camera.process_mouse_movement(xoffset, yoffset, None);
                            }
                        }
                        _ => {}
                    }
                }

                let facing_camera = rotate_view(&shader, &camera);
                if facing_camera != was_facing_camera {
                    if facing_camera {
                        music.start();
                    } else {
                        music.stop();
                    }
                    was_facing_camera = facing_camera;
                }
                Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
                model.draw(&shader);
                SDL_GL_SwapWindow(app.window);
            }
        }

        drop(model);
        drop(shader);
        drop(app);

        music.stop();
        unsafe {
            SDL_Quit();
        }

        Ok(())
    }
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

fn rotate_view(shader: &Shader, camera: &Camera) -> bool {
    unsafe {
        let mut model = Mat4::from_rotation_x(-90.0_f32.to_radians());
        let time = SDL_GetTicks() as f32 / 1000.0;
        model = model * Mat4::from_axis_angle(Vec3::Z, time);

        let model_position = model.transform_point3(Vec3::ZERO);
        let to_camera = camera.position - model_position;
        let to_camera_2d = Vec2::new(to_camera.x, to_camera.z);
        let model_forward_3d = model.transform_vector3(-Vec3::Y);
        let model_forward_2d = Vec2::new(model_forward_3d.x, model_forward_3d.z);
        let dot = model_forward_2d.normalize().dot(to_camera_2d.normalize());
        let margin = 45.0_f32.to_radians().cos();
        let facing_camera = dot >= margin;

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
        facing_camera
    }
}
