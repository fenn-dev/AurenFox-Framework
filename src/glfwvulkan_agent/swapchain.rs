use ash::{Instance, khr, vk};
use glfw::PWindow;
use colored::*;

pub struct SwapChainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapChainSupportDetails {
    pub fn new(
        physical_device: vk::PhysicalDevice,
        surface_loader: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> Self {
        unsafe {
            let capabilities = surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .expect("Failed to query surface capabilities");

            let formats = surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .expect("Failed to query surface formats");

            let present_modes = surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .expect("Failed to query surface present modes");

            Self {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }
}

pub struct AurenSwapchain {
    pub surface_khr: vk::SurfaceKHR,
    pub render_pass: vk::RenderPass,
    
    pub swapchain_khr: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub swapchain_image_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain_framebuffer: Vec<vk::Framebuffer>,
    pub swapchain_frame_buffer: Vec<vk::Framebuffer>,

    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub index_buffer: vk::Buffer,
    pub support_details: SwapChainSupportDetails,

    pub vsync_enabled: bool,
}

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

impl AurenSwapchain {
    fn choose_swap_surface_format(details: &SwapChainSupportDetails) -> vk::SurfaceFormatKHR {
        details.formats
            .iter()
            .find(|f| f.format == vk::Format::B8G8R8A8_SRGB && 
                    f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .cloned() 
            .unwrap_or_else(|| {
                details.formats.get(0)
                    .cloned()
                    .expect("Swap chain formats list is unexpectedly empty!")
            })
    }

    fn choose_swap_present_mode(vsync_enabled: bool, details: &SwapChainSupportDetails) -> vk::PresentModeKHR {
        if !vsync_enabled {
            if details.present_modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
                log_info("VSync disabled: IMMEDIATE mode");
                return vk::PresentModeKHR::IMMEDIATE;
            }
        }

        if details.present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            log_info("VSync enabled: MAILBOX mode");
            return vk::PresentModeKHR::MAILBOX;
        }

        log_info("VSync enabled: FIFO mode");
        vk::PresentModeKHR::FIFO
    }

    fn choose_swap_extent(window: &PWindow, details: &SwapChainSupportDetails) -> vk::Extent2D {
        if details.capabilities.current_extent.width != u32::MAX {
            return details.capabilities.current_extent;
        }

        let (width, height) = window.get_framebuffer_size();

        vk::Extent2D {
            width: (width as u32).clamp(
                details.capabilities.min_image_extent.width,
                details.capabilities.max_image_extent.width,
            ),
            height: (height as u32).clamp(
                details.capabilities.min_image_extent.height,
                details.capabilities.max_image_extent.height,
            ),
        }
    }

    pub fn new(window: &PWindow, instance: Instance, support_details: SwapChainSupportDetails, surface_khr: vk::SurfaceKHR, device: &ash::Device) -> Self {
        let vsync_enabled = true;

        let surface_format = Self::choose_swap_surface_format(&support_details);
        let present_mode = Self::choose_swap_present_mode(vsync_enabled, &support_details);
        let extent = Self::choose_swap_extent(&window, &support_details);

        let swapchain_image_format = surface_format.format;
        let swapchain_extent = extent;

        let mut image_count: u32 = support_details.capabilities.min_image_count + 1;
        if support_details.capabilities.max_image_count > 0 && image_count > support_details.capabilities.max_image_count {
            image_count = support_details.capabilities.max_image_count;
        }

        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, device);

        // Create Info //

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface_khr)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE) // Use EXCLUSIVE if Graphics == Present
            .pre_transform(support_details.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&create_info, None)
        };
        
        let swapchain_handle = match swapchain {
            Ok(handle) => {
                log_info("Swap chain created successfully!");
                handle
            }
            Err(e) => {
                log_err(&format!("Failed to create swap chain: {}", e));
                panic!("Vulkan Error"); 
            }
        };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain_handle)
                .expect("Failed to get swapchain images!")
        };

        // Check if it's empty (though Vulkan shouldn't allow this if creation succeeded)
        if swapchain_images.is_empty() {
            log_err("Vulkan driver returned zero swap chain images after creation!");
        }

        log_info(&format!(
            "Swap chain created successfully with {} images.", 
            swapchain_images.len()
        ));

        let mut swapchain_image_views = Vec::new();
        for &image in swapchain_images.iter() {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(swapchain_image_format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR, // Use AspectFlags, not UsageFlags
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let view = unsafe {
                device.create_image_view(&create_info, None)
                    .expect("Failed to create image view!")
            };
            swapchain_image_views.push(view);
        }

        Self {
            support_details,
            vsync_enabled,
            surface_khr,
            swapchain_images,
            swapchain_extent,
            swapchain_image_format,
            swapchain_image_views,
            swapchain_khr: swapchain_handle,
            
            swapchain_frame_buffer: Vec::new(),
            swapchain_framebuffer: Vec::new(),
            command_buffers: Vec::new(),
            
            render_pass: vk::RenderPass::null(),
            command_pool: vk::CommandPool::null(),
            index_buffer: vk::Buffer::null(),
        }
    }
}