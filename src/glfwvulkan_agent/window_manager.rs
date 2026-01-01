use glfw::{PWindow};

extern crate glfw;

pub struct AurenWindow {
    pub window: PWindow,
    pub events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub id: usize,
}

pub struct AurenWindowManager {
    pub glfw: glfw::Glfw,
    pub windows: Vec<AurenWindow>,
}

impl AurenWindowManager {
    pub fn new() -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
        
        // Good practice: Set window hints before creation
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

        AurenWindowManager {
            glfw,
            windows: Vec::new(),
        }
    }
    
    pub fn create_window(&mut self, title: &str, width: u32, height: u32, id: Option<usize>) -> Result<usize, String> {
        let (mut window, events) = self.glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .ok_or_else(|| format!("Failed to create GLFW window with title: '{}'", title))?;

        window.set_key_polling(true);

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

        self.windows.push(AurenWindow {
            window,
            events,
            title: title.to_string(),
            width,
            height,
            id: final_id,
        });

        Ok(final_id)
    }

    pub fn update(&mut self) {
        // We poll events via the glfw handle
        self.glfw.poll_events();

        for window in &mut self.windows {
            // "Draining" the events satisfies the compiler for the `events` field
            for (_, event) in glfw::flush_messages(&window.events) {
                match event {
                    glfw::WindowEvent::FramebufferSize(w, h) => {
                        // Now width and height are being "read" and updated!
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