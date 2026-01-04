use ash::{vk, khr, Device, Instance};
use colored::*;

pub struct AurenSwapchain {
    pub loader: khr::swapchain::Device,
    pub inner: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub extent: vk::Extent2D,
    pub format: vk::Format,
}

impl AurenSwapchain {
    pub unsafe fn new(
        instance: &Instance,
        phys_device: vk::PhysicalDevice,
        device: &Device,
        surface: vk::SurfaceKHR,
        surface_loader: &khr::surface::Instance,
        width: u32,
        height: u32,
        render_pass: vk::RenderPass,
    ) -> Self {
        let loader = khr::swapchain::Device::new(instance, device);
        
        // 1. Query capabilities and choose settings
        let caps = surface_loader.get_physical_device_surface_capabilities(phys_device, surface).unwrap();
        let formats = surface_loader.get_physical_device_surface_formats(phys_device, surface).unwrap();
        
        let format = formats.iter()
            .find(|f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .unwrap_or(&formats[0]);

        let extent = vk::Extent2D {
            width: width.clamp(caps.min_image_extent.width, caps.max_image_extent.width),
            height: height.clamp(caps.min_image_extent.height, caps.max_image_extent.height),
        };

        // 2. Create the Swapchain
        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(caps.min_image_count + 1)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO)
            .clipped(true);

        let inner = loader.create_swapchain(&create_info, None).expect("Failed to create swapchain");

        // 3. Get Images and Create Views
        let images = loader.get_swapchain_images(inner).unwrap();
        let image_views: Vec<vk::ImageView> = images.iter().map(|&img| {
            let view_info = vk::ImageViewCreateInfo::default()
                .image(img)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format.format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    level_count: 1,
                    layer_count: 1,
                    ..Default::default()
                });
            device.create_image_view(&view_info, None).unwrap()
        }).collect();

        // 4. Create Framebuffers
        let framebuffers: Vec<vk::Framebuffer> = image_views.iter().map(|&view| {
            let attachments = [view];
            let fb_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            device.create_framebuffer(&fb_info, None).unwrap()
        }).collect();

        println!("{} Swapchain created ({} images)", "Vulkan:".bright_green(), images.len());

        Self {
            loader,
            inner,
            images,
            image_views,
            framebuffers,
            extent,
            format: format.format,
        }
    }

    pub unsafe fn cleanup(&mut self, device: &Device) {
        for &fb in &self.framebuffers { device.destroy_framebuffer(fb, None); }
        for &view in &self.image_views { device.destroy_image_view(view, None); }
        self.loader.destroy_swapchain(self.inner, None);
    }
}