use ash::{Entry, Instance, khr, vk};
use std::ffi::CString;

pub struct AurenContext {
    pub entry: Entry, 
    pub instance: Instance,
    pub surface_loader: khr::surface::Instance,
    pub logical_device: Option<ash::Device>,
}

impl AurenContext {
    pub fn new(
        application_name: &String, 
        application_version: u32, 
        engine_name: &String, 
        engine_version: u32, 
        enabled_extension_names: &[*const i8]
    ) -> Self {
        let entry = unsafe { Entry::load().expect("Failed to load Vulkan library") };

        let app_name_cstring = CString::new(application_name.as_str())
        .expect("Application name contained interior null byte");

        let eng_name_cstring = CString::new(engine_name.as_str())
        .expect("Engine name contained interior null byte");


        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name_cstring)
            .application_version(application_version)
            .engine_name(&eng_name_cstring)
            .engine_version(engine_version)
            .api_version(vk::API_VERSION_1_3);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(enabled_extension_names);

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create Vulkan instance")
        };

        let surface_loader = khr::surface::Instance::new(&entry, &instance);

        Self {
            entry,
            instance,
            surface_loader,
            logical_device: None,
        }
    }

    pub fn create_logical_device(&mut self, physical_device: vk::PhysicalDevice) {
        let queue_family_index = unsafe {
            self.instance
                .get_physical_device_queue_family_properties(physical_device)
                .iter()
                .enumerate()
                .position(|(_, info)| info.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                .map(|index| index as u32)
                .expect("Failed to find a queue family that supports graphics")
        };

        let priorities = [1.0f32];

        let queue_info = [
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities)
        ];

        let device_features = vk::PhysicalDeviceFeatures::default();
        let extension_names = [ash::khr::swapchain::NAME.as_ptr()];

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&extension_names)
            .enabled_features(&device_features);

        let logical_device: ash::Device = unsafe {
            self.instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical device")
        };

        self.logical_device = Some(logical_device);
    }
}

impl Drop for AurenContext {
    fn drop(&mut self) {
        unsafe {
            if let Some(device) = self.logical_device.take() {
                device.destroy_device(None);
            }
            self.instance.destroy_instance(None);
        }
    }
}