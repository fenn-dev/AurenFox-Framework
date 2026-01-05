use ash::vk;

pub struct AurenRenderer {
    pub render_pass: vk::RenderPass,
    pub extent: vk::Extent2D,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub graphics_pipeline: vk::Pipeline,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,
    pub logical_device: ash::Device,
}

impl AurenRenderer {
    pub fn new(
        logical_device: ash::Device, 
        render_pass: vk::RenderPass, 
        extent: vk::Extent2D
    ) -> Self {
        // 1. Descriptor Set Layout
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
        ];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout = unsafe {
            logical_device.create_descriptor_set_layout(&layout_info, None).unwrap()
        };

        // 2. Pipeline Layout
        let layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
        let pipeline_layout = unsafe {
            logical_device.create_pipeline_layout(&pipeline_layout_info, None).unwrap()
        };

        // 3. Descriptor Pool & Set
        let pool_sizes = [
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1),
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1),
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(1)
            .pool_sizes(&pool_sizes);
        let descriptor_pool = unsafe {
            logical_device.create_descriptor_pool(&pool_info, None).unwrap()
        };

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        let descriptor_set = unsafe {
            logical_device.allocate_descriptor_sets(&alloc_info).unwrap()[0]
        };

        // Note: Real implementation would load actual SPIR-V here
        let graphics_pipeline = vk::Pipeline::null(); 

        Self {
            render_pass,
            extent,
            descriptor_set_layout,
            pipeline_layout,
            graphics_pipeline,
            descriptor_pool,
            descriptor_set,
            logical_device,
        }
    }

    pub fn update_descriptor_sets(
        &self,
        logical_device: ash::Device,
        vertex_buffer: vk::Buffer,
        scene_buffer: vk::Buffer,
    ) {
        let v_info = [vk::DescriptorBufferInfo::default()
            .buffer(vertex_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE)];

        let s_info = [vk::DescriptorBufferInfo::default()
            .buffer(scene_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE)];

        let writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(self.descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .buffer_info(&v_info),
            vk::WriteDescriptorSet::default()
                .dst_set(self.descriptor_set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .buffer_info(&s_info),
        ];

        unsafe { logical_device.update_descriptor_sets(&writes, &[]) };
    }

    pub fn create_descriptor_sets(&mut self, vertex_buffer: vk::Buffer, scene_buffer: vk::Buffer) {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
            },
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(1);

        let descriptor_pool = unsafe {
            self.logical_device.create_descriptor_pool(&pool_info, None)
                .expect("Failed to create descriptor pool!")
        };

        let layouts = [self.descriptor_set_layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = unsafe {
            self.logical_device.allocate_descriptor_sets(&alloc_info)
                .expect("Failed to allocate descriptor sets")
        };
        let descriptor_set = descriptor_sets[0];

        // Remember: these arrays must exist until update_descriptor_sets is called
        let v_info = [vk::DescriptorBufferInfo::default()
            .buffer(vertex_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE)];

        let s_info = [vk::DescriptorBufferInfo::default()
            .buffer(scene_buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE)];

        let writes = [
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .buffer_info(&v_info),
            vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .buffer_info(&s_info),
        ];

        unsafe {
            self.logical_device.update_descriptor_sets(&writes, &[]);
        }
    }
}

impl Drop for AurenRenderer {
    fn drop(&mut self) {
        unsafe {
            self.logical_device.destroy_pipeline(self.graphics_pipeline, None);
            self.logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.logical_device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}