use ash::{Instance, khr, vk};
use glfw::PWindow;

use crate::glfwvulkan_backend::helpers;

#[derive(Clone)]
pub struct AurenSwapChain {
    pub image_format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub vsync: bool,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: khr::swapchain::Device,
}

impl AurenSwapChain {
    pub fn new(
        instance: &Instance, 
        device: &ash::Device,
        window: &PWindow, 
        surface: &vk::SurfaceKHR, 
        vsync: bool, 
        details: &crate::types::SwapchainSupportDetails
    ) -> Self {
        let swapchain_loader = ash::khr::swapchain::Device::new(instance, device);

        let surface_format_khr = Self::choose_swap_surface_format(details);
        let present_mode = Self::choose_swap_present_mode(details, vsync);
        let extent = Self::choose_swap_extent(details, window);

        let mut image_count = details.capabilities.min_image_count + 1;
        if details.capabilities.max_image_count > 0 && image_count > details.capabilities.max_image_count {
            image_count = details.capabilities.max_image_count;
        }

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(*surface)
            .min_image_count(image_count)
            .image_format(surface_format_khr.format)
            .image_color_space(surface_format_khr.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(details.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain!")
        };
        Self {
            image_format: surface_format_khr,
            extent,
            vsync,
            swapchain,
            swapchain_loader,
        }
    }

    pub fn recreate(
        &mut self,
        window: &PWindow, 
        surface: &vk::SurfaceKHR, 
        vsync: bool, 
        details: &crate::types::SwapchainSupportDetails
    ) {
        unsafe {
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
        }

        let surface_format_khr = Self::choose_swap_surface_format(details);
        let present_mode = Self::choose_swap_present_mode(details, vsync);
        let extent = Self::choose_swap_extent(details, window);

        let mut image_count = details.capabilities.min_image_count + 1;
        if details.capabilities.max_image_count > 0 && image_count > details.capabilities.max_image_count {
            image_count = details.capabilities.max_image_count;
        }

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(*surface)
            .min_image_count(image_count)
            .image_format(surface_format_khr.format)
            .image_color_space(surface_format_khr.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(details.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        self.swapchain = unsafe {
            self.swapchain_loader
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain!")
        };

        self.image_format = surface_format_khr;
        self.extent = extent;

        helpers::log_info("swapchain::recreate", &format!(
            "Swap chain recreated: {}x{}", extent.width, extent.height
        ).to_string());
    }
    
    fn choose_swap_surface_format(details: &crate::types::SwapchainSupportDetails) -> vk::SurfaceFormatKHR {
        details.formats
        .iter()
        .cloned() 
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB && 
            f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| {
            details.formats[0]
        })
    }

    fn choose_swap_present_mode(
        details: &crate::types::SwapchainSupportDetails,
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
        details: &crate::types::SwapchainSupportDetails, 
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

    pub fn get_images(&self, swapchain_loader: khr::swapchain::Device, handle: vk::SwapchainKHR) -> Vec<vk::Image> {
        unsafe {
            swapchain_loader.get_swapchain_images(handle).expect("Failed to get swapchain images")
        }
    }

    pub fn create_image_views(
        &self,
        device: &ash::Device,
        images: &[vk::Image],
    ) -> Vec<vk::ImageView> {
        images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(self.image_format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                
                unsafe {
                    device.create_image_view(&create_info, None)
                        .expect("Failed to create image view")
                }
            })
            .collect()
    }
}