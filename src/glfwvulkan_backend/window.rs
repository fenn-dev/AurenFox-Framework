use ash::{Entry, Instance, vk};
use glfw::{PWindow};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

#[allow(dead_code)]
pub struct AurenWindow {
    pub window: PWindow,
    pub events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub id: usize,
    pub surface: vk::SurfaceKHR,
}

#[allow(dead_code)]
pub struct AurenWindowManager {
    pub glfw: glfw::Glfw,
    windows: Vec<AurenWindow>,
}

impl AurenWindowManager {
    pub fn new() -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

        Self {
            glfw,
            windows: Vec::new()
        }
    }

    #[allow(dead_code)]
    fn shoud_inc_id(&self, id: Option<usize>) -> Result<usize, String> {
        let final_id: usize = match id {
            Some(provided_id) => {
                if self.windows.iter().any(|w| w.id == provided_id) {
                    return Err(format!("Window ID {} is already in use!", provided_id));
                }
                provided_id
            },
            None => {
                let mut ids: Vec<usize> = self.windows.iter().map(|w| w.id).collect();
                ids.sort_unstable();

                ids.iter().enumerate()
                    .position(|(i, &id)| i != id)
                    .unwrap_or(ids.len())
            },
        };
        return Ok(final_id);
    }

    pub fn create_window(&mut self, entry: &Entry, instance: &Instance, title: &str, width: u32, height: u32, id: Option<usize>) -> Result<usize, String> {
        let (mut window, events) = self.glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .ok_or_else(|| format!("Failed to create GLFW window with title: '{}'", title))?;

        window.set_key_polling(true);

        let new_id = self.shoud_inc_id(id).expect("Window already exists");

        let surface = unsafe {
                ash_window::create_surface(
                    entry,
                    instance,
                    window.display_handle().unwrap().as_raw(),
                    window.window_handle().unwrap().as_raw(),
                    None,
                ).expect("Failed to create surface")
            };

        self.windows.push(AurenWindow {
            window,
            events,
            title: title.to_string(),
            width,
            height,
            id: new_id,
            surface,
        });

        Ok(new_id)
    }

    #[allow(dead_code)]
    pub fn update(&mut self) {
        self.glfw.poll_events();

        for window in &mut self.windows {
            for (_, event) in glfw::flush_messages(&window.events) {
                match event {
                    glfw::WindowEvent::FramebufferSize(w, h) => {
                        window.width = w as u32;
                        window.height = h as u32;
                    }
                    glfw::WindowEvent::Close => {
                        window.window.set_should_close(true);
                    }
                    _ => {}
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn check_for_id(&self, id: usize) -> bool {
        return self.windows.iter().any(|w| w.id == id)
    }

    #[allow(dead_code)]
    pub fn get_window_by_id(&mut self, id: usize) -> Option<&mut AurenWindow> {
        return self.windows.iter_mut().find(|w| w.id == id)
    }

    #[allow(dead_code)]
    pub fn destroy_window(&mut self, id: usize) {
        self.windows.retain(|win| win.id != id );
    }
}