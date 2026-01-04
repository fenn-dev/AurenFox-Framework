// Mods

mod window_manager;
mod device_manager;
mod swapchain;
mod graphics_pipeline;

// Uses
use crate::interfaces::RHI;
use ash::{vk, khr, Entry, Instance};
use colored::*;
use glfw::Context;
use window_manager::AurenWindowManager;
use device_manager::AurenDeviceManager;
use swapchain::{AurenSwapchain, SwapChainSupportDetails};
use graphics_pipeline::AurenGraphicsPipeline;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

// Structures

pub struct GLFWVulkanAgent {
    window_handler: AurenWindowManager,
    device_manager: AurenDeviceManager,
    graphics_pipeline: Option<AurenGraphicsPipeline>,
    swapchain: Option<AurenSwapchain>,

    primary_window_id: Option<usize>,
    program_should_end: bool,

    pub instance: Instance,
    pub entry: Entry,

    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
    pub framebuffer_resized: bool,
}

// Helper functions

fn log_info(msg: &str) {
    println!("{}{}->{} {} {}", 
        "Auren".bright_cyan().bold(), 
        "Fox".bright_red(), 
        "Vulkan".on_red(), 
        "[info]".bright_cyan(),
        msg
    );
}

fn log_err(msg: &str){
    panic!("{}{}->{} {} {}", 
        "Auren".bright_cyan().bold(), 
        "Fox".bright_red(), 
        "Vulkan".on_red(), 
        "[info]".bright_cyan(),
        msg
    );
}

// Implementations

impl GLFWVulkanAgent {
    pub fn new() -> Self {
        let mut window_handler = AurenWindowManager::new();
        let entry = unsafe { Entry::load().expect("Failed to load Vulkan library") };

        let required_extensions = window_handler.glfw.get_required_instance_extensions()
            .expect("Vulkan surface extensions are missing from the system");

        let extension_names_cstr: Vec<std::ffi::CString> = required_extensions
            .iter()
            .map(|ext| std::ffi::CString::new(ext.clone()).expect("Failed to convert extension name"))
            .collect();
        let extension_pointers: Vec<*const i8> = extension_names_cstr.iter().map(|ext| ext.as_ptr()).collect();

        let app_info = vk::ApplicationInfo::default()
            .application_name(unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"AurenFox App\0") })
            .api_version(vk::API_VERSION_1_3);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_pointers);

        let instance = unsafe {
            entry.create_instance(&create_info, None)
                .expect("Failed to create Vulkan instance.")
        };

        let device_manager = AurenDeviceManager::new(&instance);
        let logical_device = device_manager.logical_device.as_ref().expect("Logical device not initialized!");

        let sem_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let image_available_semaphore = unsafe { logical_device.create_semaphore(&sem_info, None).unwrap() };
        let render_finished_semaphore = unsafe { logical_device.create_semaphore(&sem_info, None).unwrap() };
        let in_flight_fence = unsafe { logical_device.create_fence(&fence_info, None).unwrap() };

        Self {
            window_handler,
            device_manager,
            graphics_pipeline: None,
            swapchain: None,
            primary_window_id: None,
            program_should_end: false,
            framebuffer_resized: false,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
            instance,
            entry,
        }
    }

    pub fn recreate_swapchain(&mut self) {
        let (width, height) = self.window_handler.windows[0].window.get_framebuffer_size();
        if width == 0 || height == 0 { return; }

        log_info("Recreating swapchain...");

        let physical_device = self.device_manager.physical_device.unwrap();
        let logical_device = self.device_manager.logical_device.as_ref().unwrap();
        let surface_loader = ash::khr::surface::Instance::new(&self.entry, &self.instance);

        if let Some(sc) = self.swapchain.as_mut() {
            unsafe {
                logical_device.device_wait_idle().expect("Wait idle failed");
                sc.recreate(
                    &self.window_handler.windows[0].window,
                    &self.instance,
                    physical_device,
                    &surface_loader,
                );
                
                // If the RenderPass changed, you would also recreate the pipeline here.
                self.framebuffer_resized = false;
            }
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

    fn draw_frame(&mut self) {
        let logical_device = self.device_manager.logical_device.as_ref().expect("No logical device");
        
        // Extract synchronization primitives
        let image_available = self.image_available_semaphore;
        let render_finished = self.render_finished_semaphore;
        let fence = self.in_flight_fence;
        let graphics_queue = self.device_manager.graphics_queue.expect("No graphics queue");
        let present_queue = self.device_manager.present_queue.expect("No present queue");

        let sc = self.swapchain.as_mut().unwrap();
        let swapchain_loader = ash::khr::swapchain::Device::new(&self.instance, logical_device);

        unsafe {
            // 1. Wait for the GPU to finish the previous frame
            logical_device.wait_for_fences(&[fence], true, u64::MAX).expect("Fence wait failed");

            // 2. Acquire an image from the swapchain
            let (image_index, _) = swapchain_loader
                .acquire_next_image(sc.swapchain_khr, u64::MAX, image_available, vk::Fence::null())
                .expect("Failed to acquire image");

            // 3. Reset the fence to busy
            logical_device.reset_fences(&[fence]).expect("Fence reset failed");

            // 4. Submit the command buffer
            let wait_semaphores = [image_available];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores = [render_finished];
            let cmd_bufs = [sc.command_buffers[image_index as usize]];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&cmd_bufs)
                .signal_semaphores(&signal_semaphores);

            logical_device.queue_submit(graphics_queue, &[submit_info], fence)
                .expect("Queue submit failed");

            // 5. Present the image back to the screen
            let swapchains = [sc.swapchain_khr];
            let indices = [image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&indices);

            swapchain_loader.queue_present(present_queue, &present_info)
                .expect("Queue present failed");
        }
    }
}

impl RHI for GLFWVulkanAgent {
    fn new(&mut self) {
        self.device_manager = AurenDeviceManager::new(&self.instance);
        self.window_handler = AurenWindowManager::new();
        self.primary_window_id = None;
    }

    fn assign_master(&mut self, id: usize) {
        self.primary_window_id = Some(id);
    }

    fn create_window(&mut self, title: &str, width: u32, height: u32, id: Option<usize>) -> Result<usize, String> {
        let window_id = self.window_handler.create_window(title, width, height, id)?;

        if self.graphics_pipeline.is_none() {
            log_info("Initializing Vulkan Swapchain and Pipeline for first window...");

            let auren_window = self.window_handler.get_window_by_id(window_id).unwrap();
            let physical_device = self.device_manager.physical_device.unwrap();
            let logical_device = self.device_manager.logical_device.as_ref().unwrap();

            // FIX: Use self.entry and self.instance instead of non-existent self.vulkan_setup
            let surface = unsafe {
                ash_window::create_surface(
                    &self.entry,
                    &self.instance,
                    auren_window.window.display_handle().unwrap().as_raw(),
                    auren_window.window.window_handle().unwrap().as_raw(),
                    None,
                ).expect("Failed to create surface")
            };

            let surface_loader = ash::khr::surface::Instance::new(&self.entry, &self.instance);
            let support_details = SwapChainSupportDetails::new(physical_device, &surface_loader, surface);

            let mut sc = AurenSwapchain::new(
                &auren_window.window,
                self.instance.clone(),
                logical_device.clone(),
                support_details,
                surface,
                logical_device,
                0,
            );

            sc.create_image_views().unwrap();
            sc.create_render_pass();
            sc.create_framebuffers();
            sc.create_command_buffers(0); // Using index 0 from device_manager

            let gp = AurenGraphicsPipeline::new(
                logical_device.clone(),
                sc.render_pass,
                sc.swapchain_extent,
                true,
                true
            );

            // Record initial commands
            unsafe { sc.record_commands(&gp); }

            let vertices = [
                Vertex { pos: [0.0, -0.5, 0.0], _pad: 0.0, color: [1.0, 0.0, 0.0, 1.0] },
                Vertex { pos: [0.5, 0.5, 0.0], _pad: 0.0, color: [0.0, 1.0, 0.0, 1.0] },
                Vertex { pos: [-0.5, 0.5, 0.0], _pad: 0.0, color: [0.0, 0.1, 1.0, 1.0] },
            ];

            // Identity matrix for the SceneData
            let scene_data = SceneData {
                model_matrix: [
                    1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 1.0,
                ],
                base_color: [1.0, 1.0, 1.0, 1.0],
                time: 0.0,
                _padding: [0.0; 3],
            };

            // Create the actual buffers (Note: You'll need a buffer creation helper)
            let (v_buffer, v_memory) = create_gpu_buffer(&logical_device, &vertices, vk::BufferUsageFlags::STORAGE_BUFFER);
            let (s_buffer, s_memory) = create_gpu_buffer(&logical_device, &[scene_data], vk::BufferUsageFlags::UNIFORM_BUFFER);

            self.swapchain = Some(sc);
            self.graphics_pipeline = Some(gp);
        }

        Ok(window_id)
    }

    fn start_frame(&mut self) {
        self.cleanup_closed_windows();
        if self.get_window_count() == 0 {
            self.program_should_end = true;
            return;
        }
        if self.primary_window_id.is_some_and(|id| !self.window_handler.check_for_id(id)) {
            self.program_should_end = true;
            return;
        }
        self.window_handler.update();
    }

    fn end_frame(&mut self) {
        if self.program_should_end { return; }

        // Just check if we are ready to draw, then call the dedicated function
        if self.swapchain.is_some() && self.graphics_pipeline.is_some() {
            self.draw_frame(); 
        }
    }

    fn destroy_window(&mut self, id : usize) {
        self.window_handler.destroy_window(id);
    }

    fn should_close(&self) -> bool {
        return self.program_should_end;
    }
}