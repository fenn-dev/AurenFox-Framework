use std::ffi::CStr;

use ash::{Instance, vk::{self, QueueFamilyProperties}};

#[derive(Debug, Clone)]
pub struct PhysicalDeviceDetails {
    pub handle: vk::PhysicalDevice,
    pub name: String,
    pub device_type: vk::PhysicalDeviceType,
    pub graphics_index: Option<u32>,
    pub vram_mb: u64,
    pub score: i32,
    pub queue_families: Vec<QueueFamilyProperties>,
}

pub struct AurenDevice {
    pub devices: Vec<PhysicalDeviceDetails>,
}

impl AurenDevice {
    pub fn new(instance: &Instance) -> Self {
        let mut slf = Self { devices: Vec::new() };
        slf.reload_devices(instance);
        slf
    }

    pub fn reload_devices(&mut self, instance: &Instance) {
        let physical_devices = unsafe {
            instance.enumerate_physical_devices()
            .expect("Failed to find GPU's with Vulkan Support")
        };

        self.devices = physical_devices.into_iter().map(|device| {
            let props = unsafe { instance.get_physical_device_properties(device) };
            let mem_props = unsafe { instance.get_physical_device_memory_properties(device) };

            let name = unsafe {
                CStr::from_ptr(props.device_name.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            };

            let vram_mb = mem_props.memory_heaps
                .iter()
                .take(mem_props.memory_heap_count as usize)
                .filter(|heap| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
                .map(|heap| heap.size)
                .sum::<u64>() / 1024 / 1024;

            let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device) };
            let graphics_index = queue_families.iter().enumerate()
                .find(|(_, info)| info.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                .map(|(i, _)| i as u32);

            let mut score = vram_mb as i32;
            if props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                score += 1000;
            }

            PhysicalDeviceDetails {
                handle: device,
                name,
                device_type: props.device_type,
                vram_mb,
                score,
                graphics_index,
                queue_families,
            }
        }).collect();
    }

    pub fn get_optimal_device(&self) -> Option<&PhysicalDeviceDetails> {
        self.devices.iter().max_by_key(|d| d.score)
    }

    pub fn print_all_devices(&self) {
        println!("--- Available Vulkan Devices ---");
        for (i, dev) in self.devices.iter().enumerate() {
            println!("[{}] {} ({:?}) - {} MB VRAM | Score: {}",
                i, dev.name, dev.device_type, dev.vram_mb, dev.score);
        }
    }
}
