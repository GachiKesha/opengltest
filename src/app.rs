use glfw::ffi::{
    GLFW_CONTEXT_VERSION_MAJOR, GLFW_CONTEXT_VERSION_MINOR, GLFW_CURSOR, GLFW_CURSOR_DISABLED,
    GLFW_CURSOR_NORMAL, GLFW_OPENGL_CORE_PROFILE, GLFW_OPENGL_PROFILE, GLFWwindow,
    glfwCreateWindow, glfwGetFramebufferSize, glfwGetProcAddress, glfwGetWindowUserPointer,
    glfwInit, glfwMakeContextCurrent, glfwSetFramebufferSizeCallback, glfwSetInputMode,
    glfwSetScrollCallback, glfwTerminate, glfwWindowHint,
};

pub struct App {
    pub width: i32,
    pub height: i32,
    pub title: &'static str,
    pub window: *mut GLFWwindow,
}

impl App {
    pub fn new(width: i32, height: i32, title: &'static str) -> Self {
        unsafe {
            glfwInit();
            glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
            glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 6);
            glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
        }
        Self {
            width,
            height,
            title,
            window: std::ptr::null_mut(),
        }
    }

    pub fn run(&mut self) -> Result<(), &'static str> {
        unsafe {
            let title = std::ffi::CString::new(self.title).unwrap();
            self.window = glfwCreateWindow(
                self.width,
                self.height,
                title.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            if self.window.is_null() {
                glfwTerminate();
                return Err("Failed to create GLFW window");
            }

            glfwMakeContextCurrent(self.window);

            glad_gl::gl::load(|name| {
                let name = std::ffi::CString::new(name).unwrap();
                let ptr = glfwGetProcAddress(name.as_ptr())
                    .map_or(std::ptr::null(), |p| p as *const std::ffi::c_void);
                if ptr.is_null() {
                    eprintln!("GL function not found: {name:?}");
                }
                ptr
            });

            let mut framebuffer_width = 0;
            let mut framebuffer_height = 0;

            glfwGetFramebufferSize(self.window, &mut framebuffer_width, &mut framebuffer_height);
            glad_gl::gl::Viewport(0, 0, framebuffer_width, framebuffer_height);

            glfwSetFramebufferSizeCallback(self.window, Some(framebuffer_size_callback));
            glfwSetScrollCallback(self.window, Some(scroll_callback));
        }

        Ok(())
    }

    pub fn set_cursor_captured(&self, captured: bool) {
        unsafe {
            glfwSetInputMode(
                self.window,
                GLFW_CURSOR,
                if captured {
                    GLFW_CURSOR_DISABLED
                } else {
                    GLFW_CURSOR_NORMAL
                },
            );
        }
    }
}

unsafe extern "C" fn framebuffer_size_callback(_window: *mut GLFWwindow, width: i32, height: i32) {
    unsafe {
        glad_gl::gl::Viewport(0, 0, width, height);
    }
}

unsafe extern "C" fn scroll_callback(window: *mut GLFWwindow, _xoffset: f64, yoffset: f64) {
    unsafe {
        let scroll_y = glfwGetWindowUserPointer(window) as *mut f64;
        if !scroll_y.is_null() {
            *scroll_y += yoffset;
        }
    }
}
