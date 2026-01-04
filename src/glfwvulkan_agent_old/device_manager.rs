use ash::{Instance, vk};
use colored::*;

pub struct AurenDeviceManager {
    pub physical_device: Option<vk::PhysicalDevice>,
    pub logical_device: Option<ash::Device>,
    pub graphics_queue: Option<vk::Queue>,
    pub present_queue: Option<vk::Queue>,
}

impl AurenDeviceManager {
    pub fn new_empty() -> Self {
        Self {
            physical_device: None,
            logical_device: None,
            graphics_queue: None,
            present_queue: None,
        }
    }

    pub fn new(instance: &Instance) -> Self {
        let physical_device_list = unsafe {
            instance.enumerate_physical_devices()
                .expect("Failed to enumerate physical devices")
        };

        let physical_device = *physical_device_list.first()
            .expect("No GPUs found supporting Vulkan");

        let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
        let priorities = [1.0_f32];

        // Note: For a production engine, you would query indices properly.
        // Assuming index 0 for both graphics and present for now.
        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(0)
            .queue_priorities(&priorities);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extensions);

        let logical_device = unsafe {
            instance.create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical device")
        };

        let graphics_queue = unsafe { logical_device.get_device_queue(0, 0) };

        Self {
            physical_device: Some(physical_device),
            logical_device: Some(logical_device),
            graphics_queue: Some(graphics_queue),
            present_queue: Some(graphics_queue),
        }
    }

    #[allow(dead_code)]
    pub fn check_for_device_loss(&self) -> bool {
        false
    }
    
    #[allow(dead_code)]
    pub fn get_physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device.unwrap()
    }

    #[allow(dead_code)]
    pub fn get_logical_device(&self) -> &ash::Device {
        self.logical_device.as_ref().expect("Logical device not initialized")
    }
}