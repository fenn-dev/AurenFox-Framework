use ash::{vk, Entry, Instance, khr};
use vk::EXT_DEBUG_UTILS_NAME as NAME;
use vk::EXT_DEBUG_UTILS_SPEC_VERSION as SPEC_VERSION;

pub struct AurenVulkanSetup {
    pub instance: Instance,
}

impl AurenVulkanSetup {
    pub fn new() -> Self {
        let entry = unsafe { Entry::load().expect("Failed to load Vulkan library") };

        let app_info = vk::ApplicationInfo::default()
            .application_name(unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"AurenFox App\0") })
            .application_version(0)
            .engine_name(unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"AurenFox Engine\0") })
            .engine_version(0)
            .api_version(vk::API_VERSION_1_3);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info);

        let instance = unsafe {
            Entry::load().expect("Vulkan library not found!")
                .create_instance(&create_info, None)
                .expect("Failed to create Vulkan instance")
        };

        Self {
            instance,
        }
    }
}