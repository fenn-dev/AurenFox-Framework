use ash::vk;
use std::ptr::copy_nonoverlapping;

pub struct AurenBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
}

impl AurenBuffer {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Self {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.create_buffer(&buffer_info, None).expect("Failed to create buffer") };

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let mem_type_index = Self::find_memory_type(
            instance,
            physical_device,
            mem_requirements.memory_type_bits,
            properties);
        
        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type_index);

        let memory = unsafe { device.allocate_memory(&alloc_info, None).expect("Failed to allocate buffer memory") };

        unsafe { device.bind_buffer_memory(buffer, memory, 0).expect("Failed to bind buffer memory") };

        Self { buffer, memory, size }
    }

    fn find_memory_type(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> u32 {
        let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        for i in 0..mem_properties.memory_type_count {
            if (type_filter & (1 << i)) != 0
                && (mem_properties.memory_types[i as usize].property_flags & properties) == properties
            {
                return i;
            }
        }
        panic!("Failed to find suitable memory type!")
    }

    pub fn upload_data<T: Copy>(&self, device: &ash::Device, data: &[T]) {
        let size = (data.len() * std::mem::size_of::<T>()) as vk::DeviceSize;
        unsafe {
            let data_ptr = device
                .map_memory(self.memory, 0, size, vk::MemoryMapFlags::empty())
                .expect("Failed to map memory");
            copy_nonoverlapping(data.as_ptr(), data_ptr as *mut T, data.len());
            device.unmap_memory(self.memory);
        }
    }

    
}