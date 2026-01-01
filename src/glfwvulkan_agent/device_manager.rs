use colored::*;
use ash::{Instance, khr, vk::{self}};

pub struct AurenDeviceManager {
    physical_device: Option<vk::PhysicalDevice>,
    logical_device: Option<ash::Device>,
}

impl AurenDeviceManager {
    pub fn new_empty() -> Self {
        Self {
            physical_device: None,
            logical_device: None,
        }
    }

    pub fn new(instance: &Instance) -> Self {
        let physical_device_list = unsafe {
            instance.enumerate_physical_devices().unwrap()
        };

        let physical_device = physical_device_list.first().expect("No GPUs found");

        let _device_extensions = [
            khr::swapchain::NAME.as_ptr() as *const i8,
        ];

        let _device_features = vk::PhysicalDeviceFeatures::default();

        let priorities = [1.0_f32];

        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(0)
            .queue_priorities(&priorities);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info));

        let logical_device = unsafe {
            instance.create_device(*physical_device, &device_create_info, None)
                .unwrap_or_else(|_| {
                    panic!("{}{}->{} {} {}", 
                        "Auren".bright_cyan().bold(), 
                        "Fox".bright_red(), 
                        "Vulkan".on_red(), 
                        "[err]".bright_white().on_red().bold(),
                        "Couldn't create logical device"
                    )
                })
        };

        Self {
            physical_device: Some(*physical_device),
            logical_device: Some(logical_device),
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
        self.logical_device.as_ref().unwrap()
    }
}