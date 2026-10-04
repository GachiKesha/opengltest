use glad_gl::gl::{GLboolean, TRUE};
use glam::{Mat4, Vec3};

// Defines several possible options for camera movement. Used as abstraction to stay away from window-system specific input methods
pub enum CameraMovement {
    Forward,
    Backward,
    Left,
    Right,
}

// Default camera values
const YAW: f32 = 180.0;
const PITCH: f32 = -15.0;
const SPEED: f32 = 50.0;
const SENSITIVITY: f32 = 0.1;
const ZOOM: f32 = 45.0;

pub struct Camera {
    // camera Attributes
    pub position: Vec3,
    pub front: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub world_up: Vec3,
    // euler Angles
    pub yaw: f32,
    pub pitch: f32,
    // camera options
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    pub zoom: f32,
}

impl Camera {
    // constructor with vectors
    pub fn new(position: Vec3, up: Vec3, yaw: f32, pitch: f32) -> Self {
        let mut camera = Self {
            position,
            front: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::default(),
            right: Vec3::default(),
            world_up: up,
            yaw,
            pitch,
            movement_speed: SPEED,
            mouse_sensitivity: SENSITIVITY,
            zoom: ZOOM,
        };

        camera.update_camera_vectors();
        camera
    }

    // returns the view matrix calculated using Euler Angles and the LookAt Matrix
    pub fn get_view_matrix(&self) -> Mat4 {
        glam::camera::rh::view::look_at_mat4(self.position, self.position + self.front, self.up)
    }
    // processes input received from any keyboard-like input system. Accepts input parameter in the form of camera defined ENUM (to abstract it from windowing systems)
    pub fn process_keyboard(&mut self, direction: CameraMovement, delta_time: f32) {
        let velocity = self.movement_speed * delta_time;
        match direction {
            CameraMovement::Forward => self.position += self.front * velocity,
            CameraMovement::Backward => self.position -= self.front * velocity,
            CameraMovement::Left => self.position -= self.right * velocity,
            CameraMovement::Right => self.position += self.right * velocity,
        }
    }

    // processes input received from a mouse input system. Expects the offset value in both the x and y direction.
    pub fn process_mouse_movement(
        &mut self,
        xoffset: f32,
        yoffset: f32,
        constrain_pitch: Option<GLboolean>,
    ) {
        let xoffset = xoffset * self.mouse_sensitivity;
        let yoffset = yoffset * self.mouse_sensitivity;

        self.yaw += xoffset;
        self.pitch += yoffset;

        // make sure that when pitch is out of bounds, screen doesn't get flipped
        if constrain_pitch.unwrap_or(TRUE) == TRUE {
            self.pitch = self.pitch.clamp(-89.0, 89.0);
        }

        // update Front, Right and Up Vectors using the updated Euler angles
        self.update_camera_vectors();
    }

    // processes input received from a mouse scroll-wheel event. Only requires input on the vertical wheel-axis
    pub fn process_mouse_scroll(&mut self, yoffset: f32) {
        self.zoom -= yoffset;
        if self.zoom < 1.0 {
            self.zoom = 1.0;
        }
        if self.zoom > 45.0 {
            self.zoom = 45.0;
        }
    }

    // calculates the front vector from the Camera's (updated) Euler Angles
    fn update_camera_vectors(&mut self) {
        let yaw = self.yaw.to_radians();
        let pitch = self.pitch.to_radians();
        // calculate the new Front vector
        let front = Vec3 {
            x: yaw.cos() * pitch.cos(),
            y: pitch.sin(),
            z: yaw.sin() * pitch.cos(),
        };
        self.front = front.normalize();
        // also re-calculate the Right and Up vector
        self.right = self.front.cross(self.world_up).normalize(); // normalize the vectors, because their length gets closer to 0 the more you look up or down which results in slower movement.
        self.up = self.right.cross(self.front).normalize();
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Vec3::ZERO, Vec3::Y, YAW, PITCH)
    }
}
