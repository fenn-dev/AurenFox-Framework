use ash::{amd::display_native_hdr::Device, vk};

pub struct AurenRenderer {
    pub extent: vk::Extent2D,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub graphics_pipeline: vk::Pipeline,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,
    pub logical_device: ash::Device,
    pub render_pass: Option<vk::RenderPass>,
    pub frame_buffers: Vec<vk::Framebuffer>,
}

impl AurenRenderer {
    pub fn new(
        logical_device: ash::Device, 
        extent: vk::Extent2D,
        surface_format: vk::Format,
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

        let graphics_pipeline = vk::Pipeline::null(); 


        Self {
            extent,
            descriptor_set_layout,
            pipeline_layout,
            graphics_pipeline,
            descriptor_pool,
            descriptor_set,
            logical_device,
            render_pass: None,
            frame_buffers: Vec::new(),
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

    pub fn create_render_pass(&mut self, surface_format: vk::Format, device: &ash::Device) -> vk::RenderPass {

        let color_attachment = vk::AttachmentDescription::default()
            .format(surface_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let attachments = [color_attachment];

        let color_attachment_refs = [
            vk::AttachmentReference::default()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        ];

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs);

        let subpasses = [subpass];

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dependency_flags(vk::DependencyFlags::empty());

        let dependencies = [dependency];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        return unsafe {
            device.create_render_pass(&render_pass_info, None)
            .expect("Failed to create render pass!")
        };
    }

    pub fn create_framebuffers(
        &mut self,
        device: &ash::Device,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
        image_views: &[vk::ImageView],
    ) {
        self.frame_buffers = image_views
            .iter()
            .map(|&view| {
                let attachments = [view];
                let create_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(swapchain_extent.width)
                    .height(swapchain_extent.height)
                    .layers(1);

                unsafe {
                    device.create_framebuffer(&create_info, None)
                        .expect("Failed to create framebuffer")
                }
        }).collect();
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