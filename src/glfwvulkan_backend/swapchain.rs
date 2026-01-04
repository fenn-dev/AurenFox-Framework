use ash::{Instance, vk};
use glfw::PWindow;

pub struct AurenSwapChain {
    instance: Instance,
    logical_device: vk::Device,
    surface: vk::SurfaceKHR,
    image_format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    vsync: bool,
}

impl AurenSwapChain {
    pub fn new(
        instance: &Instance, 
        device: &ash::Device, // You need the ash logical device
        window: &PWindow, 
        surface: vk::SurfaceKHR, 
        vsync: bool, 
        details: &crate::types::swapchain_support_details
    ) -> Self {
        // 1. Use the Device loader for the swapchain
        let swapchain_loader = ash::khr::swapchain::Device::new(instance, device);

        let surface_format = Self::choose_swap_surface_format(details);
        let present_mode = Self::choose_swap_present_mode(details, vsync);
        let extent = Self::choose_swap_extent(details, window);

        // 2. image_count MUST be mutable
        let mut image_count = details.capabilities.min_image_count + 1;
        if details.capabilities.max_image_count > 0 && image_count > details.capabilities.max_image_count {
            image_count = details.capabilities.max_image_count;
        }

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(details.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        // 3. The actual creation call
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .expect("Failed to create swap chain!")
        };

        // 4. Return the struct
        Self {
            instance: instance.clone(),
            logical_device: device.handle(), // or however you store your device handle
            surface,
            image_format: surface_format,
            extent,
            vsync,
        }
    }
    
    fn choose_swap_surface_format(details: &crate::types::swapchain_support_details) -> vk::SurfaceFormatKHR {
        details.formats
        .iter()
        .cloned() // Ash handles are Copy, but if using a vec, we clone the struct
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB && 
            f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| {
            // If the list is empty, this would panic, matching your logic
            details.formats[0]
        })
    }

    fn choose_swap_present_mode(
        details: &crate::types::swapchain_support_details,
        vsync_enabled: bool,
    ) -> vk::PresentModeKHR {
        if !vsync_enabled {
            if details.present_modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
                println!("Swap chain: Using Immediate mode (No V-Sync, Maximum FPS).");
                return vk::PresentModeKHR::IMMEDIATE;
            }
        }

        for &mode in &details.present_modes {
            if mode == vk::PresentModeKHR::MAILBOX {
                println!("Swap chain: Using Mailbox mode (Triple Buffered V-Sync).");
                return vk::PresentModeKHR::MAILBOX;
            }
        }

        println!("Swap chain: Using FIFO mode (Standard V-Sync).");
        vk::PresentModeKHR::FIFO
    }

    fn choose_swap_extent(
        details: &crate::types::swapchain_support_details, 
        window: &glfw::Window
    ) -> vk::Extent2D {
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
}