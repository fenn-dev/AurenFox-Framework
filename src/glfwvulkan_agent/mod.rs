// Mods

mod window_manager;
mod device_manager;
mod vulkan_setup;
mod swapchain;

// Uses

use crate::interfaces::RHI;
use glfw::Context;
use window_manager::AurenWindowManager;
use device_manager::AurenDeviceManager;
use vulkan_setup::AurenVulkanSetup;
use swapchain::AurenSwapchain;

// Structures

pub struct GLFWVulkanAgent {
    vulkan_setup: AurenVulkanSetup,
    window_handler: AurenWindowManager,
    device_manager: AurenDeviceManager,
    swapchain: AurenSwapchain,

    primary_window_id: Option<usize>,
    program_should_end: bool,
}


// Implementations

impl GLFWVulkanAgent {
    pub fn new() -> Self {
        let program_should_end = false;
        Self {
            vulkan_setup: AurenVulkanSetup::new(),
            window_handler: AurenWindowManager::new(),
            device_manager: AurenDeviceManager::new_empty(),
            swapchain: AurenSwapchain::new(),

            primary_window_id: None,
            program_should_end,
        }
    }

    fn swap_buffers(&mut self) {
        for window in &mut self.window_handler.windows {
            window.window.swap_buffers();
        }
    }

    fn get_window_count(&self) -> usize {
        self.window_handler.windows.len()
    }

    #[allow(dead_code)]
    fn get_window_title(&self, index: usize) -> Option<&str> {
        if index < self.window_handler.windows.len() {
            Some(&self.window_handler.windows[index].title)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn make_current(&mut self, index: usize) {
        if index < self.window_handler.windows.len() {
            self.window_handler.windows[index]
                .window
                .make_current();
        }
    }

    fn cleanup_closed_windows(&mut self) {
        // This looks at every window and only keeps it if should_close is FALSE
        self.window_handler.windows.retain(|w| {
            !w.window.should_close()
        });
    }
}

impl RHI for GLFWVulkanAgent {
    fn new(&mut self) {
        self.vulkan_setup = AurenVulkanSetup::new();
        self.device_manager = AurenDeviceManager::new(&self.vulkan_setup.instance);
        self.window_handler = AurenWindowManager::new();
        self.primary_window_id = None;
    }

    fn assign_master(&mut self, id: usize) {
        self.primary_window_id = Some(id);
    }

    fn create_window(&mut self, title: &str, width: u32, height: u32, id: Option<usize>) -> Result<usize, String> {
        self.window_handler.create_window(title, width, height, id)
    }

    fn start_frame(&mut self) {
        self.cleanup_closed_windows();

        let count = self.get_window_count();
            if count == 0 {
                self.program_should_end = true;
                return;
            }

            // Check if the master window is set
            if self.primary_window_id.is_some_and(|id| !self.window_handler.check_for_id(id)) {
                self.program_should_end = true;
                return;
            }

        self.window_handler.update();
    }

    fn end_frame(&mut self) {
        if self.program_should_end { return; }
        self.swap_buffers();
    }

    fn destroy_window(&mut self, id : usize) {
        self.window_handler.destroy_window(id);
    }

    fn should_close(&self) -> bool {
        return self.program_should_end;
    }
}