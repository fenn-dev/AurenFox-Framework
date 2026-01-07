use std::ffi::CString;
use ash::vk;

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

    pub compute_pipeline_layout: vk::PipelineLayout,
    pub compute_pipeline: vk::Pipeline,
}

pub struct AurenSync {
    pub image_available: vk::Semaphore,
    pub render_finished: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
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

        let layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
        let pipeline_layout = unsafe {
            logical_device.create_pipeline_layout(&pipeline_layout_info, None).unwrap()
        };

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


        Self {
            extent,
            descriptor_set_layout,
            pipeline_layout,
            graphics_pipeline: vk::Pipeline::null(),
            descriptor_pool,
            descriptor_set,
            logical_device,
            render_pass: None,
            frame_buffers: Vec::new(),
            compute_pipeline_layout: vk::PipelineLayout::null(),
            compute_pipeline: vk::Pipeline::null(),
        }
    }

    pub fn create_shader_module(&self, code: Vec<u8>) -> vk::ShaderModule {
        let shader_info = vk::ShaderModuleCreateInfo::default()
            .code(unsafe {
                let (prefix, code_u32, suffix) = code.align_to::<u32>();
                if !prefix.is_empty() || !suffix.is_empty() {
                    panic!("Shader code alignment error");
                }
                code_u32
            });

        unsafe {
            self.logical_device
                .create_shader_module(&shader_info, None)
                .expect("Failed to create shader module")
        }
    }

    pub fn create_graphics_pipeline(
        &mut self,
        vertex_shader: vk::ShaderModule,
        fragment_shader: vk::ShaderModule,
    ) {
        let entry_name = CString::new("main").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_shader)
                .name(&entry_name),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_shader)
                .name(&entry_name),
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewports = [vk::Viewport::default()
            .width(self.extent.width as f32)
            .height(self.extent.height as f32)
            .max_depth(1.0)];

        let scissors = [vk::Rect2D::default()
            .offset(vk::Offset2D {x: 0, y: 0})
            .extent(self.extent)];

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(self.pipeline_layout)
            .render_pass(self.render_pass.expect("Missing render_pass"))
            .subpass(0);

        self.graphics_pipeline = unsafe {
            self.logical_device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .expect("Failed to create graphics pipeline")[0]
        };

        self.render_pass = Some(self.render_pass.expect("Missing render_pass"));
    }

    pub fn create_compute_pipeline(&mut self, compute_module: vk::ShaderModule) {
        let entry_name = std::ffi::CString::new("main").unwrap();

        let stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(compute_module)
            .name(&entry_name);

        let create_info = vk::ComputePipelineCreateInfo::default()
            .stage(stage_info)
            .layout(self.pipeline_layout);

        self.compute_pipeline = unsafe {
            self.logical_device.create_compute_pipelines(vk::PipelineCache::null(), &[create_info], None)
            .expect("Failed to create compute pipeline")[0]
        };
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

        let render_pass = unsafe {
        device.create_render_pass(&render_pass_info, None)
            .expect("Failed to create render pass")
        };
        
        self.render_pass = Some(render_pass);

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

    pub fn create_sync_objects(device: &ash::Device) -> AurenSync {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default()
            .flags(vk::FenceCreateFlags::SIGNALED);
        
        unsafe  {
            AurenSync {
                image_available: device.create_semaphore(&semaphore_info, None).unwrap(),
                render_finished: device.create_semaphore(&semaphore_info, None).unwrap(),
                in_flight_fence: device.create_fence(&fence_info, None).unwrap(),
            }
        }
    }

    pub fn record_commands(
        &self,
        cmd: vk::CommandBuffer,
        framebuffer: vk::Framebuffer,
        _vertex_buffer: vk::Buffer,
        vertex_count: u32,
    ) {
        let begin_info = vk::CommandBufferBeginInfo::default();

        unsafe {
            self.logical_device.begin_command_buffer(cmd, &begin_info).unwrap();

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue { float32: [0.02, 0.02, 0.02, 1.0] },
            }];

            let render_pass_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.render_pass.unwrap())
                .framebuffer(framebuffer)
                .render_area(self.extent.into())
                .clear_values(&clear_values);

            self.logical_device.cmd_begin_render_pass(cmd, &render_pass_info, vk::SubpassContents::INLINE);
            self.logical_device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline);

            // This links your DescriptorSet (Storage + Uniform) to the shader
            self.logical_device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_set],
                &[],
            );

            // Draw!
            self.logical_device.cmd_draw(cmd, vertex_count, 1, 0, 0);

            self.logical_device.cmd_end_render_pass(cmd);
            self.logical_device.end_command_buffer(cmd).unwrap();
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