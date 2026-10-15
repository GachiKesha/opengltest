use sdl3::sys::{
    error::SDL_GetError,
    init::{SDL_INIT_VIDEO, SDL_Init, SDL_Quit},
    mouse::SDL_SetWindowRelativeMouseMode,
    video::{
        SDL_CreateWindow, SDL_DestroyWindow, SDL_GL_CONTEXT_MAJOR_VERSION,
        SDL_GL_CONTEXT_MINOR_VERSION, SDL_GL_CONTEXT_PROFILE_CORE, SDL_GL_CONTEXT_PROFILE_MASK,
        SDL_GL_CreateContext, SDL_GL_DOUBLEBUFFER, SDL_GL_GetProcAddress, SDL_GL_SetAttribute,
        SDL_WINDOW_OPENGL, SDL_WINDOW_RESIZABLE, SDL_Window,
    },
};

pub struct App {
    pub width: i32,
    pub height: i32,
    pub title: &'static str,
    pub window: *mut SDL_Window,
}

impl App {
    pub fn new(width: i32, height: i32, title: &'static str) -> Self {
        unsafe {
            if SDL_Init(SDL_INIT_VIDEO) == false {
                eprint!("SDL_Init Error: {}", *SDL_GetError());
            }

            SDL_GL_SetAttribute(
                SDL_GL_CONTEXT_PROFILE_MASK,
                SDL_GL_CONTEXT_PROFILE_CORE.value(),
            );
            SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
            SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 6);
            SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
        }
        Self {
            width,
            height,
            title,
            window: std::ptr::null_mut(),
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        unsafe {
            let title = std::ffi::CString::new(self.title).unwrap();
            self.window = SDL_CreateWindow(
                title.as_ptr(),
                self.width,
                self.height,
                SDL_WINDOW_OPENGL | SDL_WINDOW_RESIZABLE,
            );

            if self.window.is_null() {
                SDL_Quit();
                return Err("Failed to create GLFW window".to_string());
            }

            let context = SDL_GL_CreateContext(self.window);
            if context.is_null() {
                SDL_DestroyWindow(self.window);
                SDL_Quit();
                return Err(format!(
                    "Failed to create OpenGL context: {}",
                    *SDL_GetError(),
                ));
            }

            glad_gl::gl::load(|name| {
                let name = std::ffi::CString::new(name).unwrap();
                let ptr = SDL_GL_GetProcAddress(name.as_ptr())
                    .map_or(std::ptr::null(), |p| p as *const std::ffi::c_void);
                if ptr.is_null() {
                    eprintln!("GL function not found: {name:?}");
                }
                ptr
            });

            glad_gl::gl::Viewport(0, 0, self.width, self.height);
        }

        Ok(())
    }

    pub fn set_cursor_captured(&self, captured: bool) {
        unsafe {
            SDL_SetWindowRelativeMouseMode(self.window, captured);
        }
    }
}
