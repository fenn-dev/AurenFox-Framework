use std::{fs, u64};
use ash::vk;
use aurenfox::{glfwvulkan_backend::{
    buffers::AurenBuffer, context::AurenContext, device::AurenDevice, helpers, renderer::AurenRenderer, swapchain::AurenSwapChain, types::SwapchainSupportDetails, window::AurenWindowManager
}, types::{SceneData, Vertex}};

fn main() {
    helpers::log_info("Main", "Initializing");

    let mut window_manager = AurenWindowManager::new();

    let required_extensions = window_manager.glfw.get_required_instance_extensions()
    .expect("GLFW: Failed to get required instance extensions");

    // 1. Create CStrings and KEEP THEM in a variable so they don't drop
    let extension_names_c: Vec<std::ffi::CString> = required_extensions
        .iter()
        .map(|ext| std::ffi::CString::new(ext.as_str()).unwrap())
        .collect();

    // 2. Now get the pointers from the CStrings
    let extension_ptrs: Vec<*const i8> = extension_names_c
        .iter()
        .map(|c_str| c_str.as_ptr())
        .collect();

    // 3. Pass them to your context
    let mut context = AurenContext::new(
        &"AurenFox App".to_string(), 1,
        &"AurenFox Engine".to_string(), 1,
        &extension_ptrs
    );

    // -- 2. Create Window FIRST --
    // We need the surface to verify GPU compatibility
    window_manager.create_window(&context.entry, &context.instance, "AurenFox Window", 720, 480, Some(0))
        .expect("Failed to create window");

    let window_entry = window_manager.get_window_by_id(0).expect("Err");

    // -- 3. Pick GPU & Create Logical Device --
    let device_manager = AurenDevice::new(&context.instance);
    let optimal_gpu = device_manager.get_optimal_device().expect("No GPU found");
    
    // Pass the surface to the device creation if you update context.rs later to be safer
    context.create_logical_device(optimal_gpu.handle);

    // -- 4. Setup Swapchain --
    let details = SwapchainSupportDetails::new(
        optimal_gpu.handle, 
        &context.surface_loader, 
        window_entry.surface
    );

    let swapchain = AurenSwapChain::new(
        &context.instance, 
        context.logical_device.as_ref().expect("err"), 
        &window_entry.window,
        &window_entry.surface,
        true,
        &details
    );

    // -- 5. Initialize Renderer --
    let device = context.logical_device.as_ref().expect("err");
    let mut renderer = AurenRenderer::new(
        device.clone(), 
        swapchain.extent, 
        swapchain.image_format.format
    );

    renderer.create_render_pass(swapchain.image_format.format, device);

    let vert_spv = std::fs::read("src/shaders/vert.spv").expect("Fix: run dxc for vertex");
    let frag_spv = std::fs::read("src/shaders/frag.spv").expect("Fix: run dxc for fragment");
    let comp_spv = std::fs::read("src/shaders/comp.spv").expect("Fix: run dxc for compute");

    let v_mod = renderer.create_shader_module(vert_spv);
    let f_mod = renderer.create_shader_module(frag_spv);
    let c_mod = renderer.create_shader_module(comp_spv);

    renderer.create_graphics_pipeline(v_mod, f_mod);
    renderer.create_compute_pipeline(c_mod);

    let swapchain_images = swapchain.get_images(swapchain.swapchain_loader.clone(), swapchain.swapchain);
    let swapchain_image_views = swapchain.create_image_views(device, &swapchain_images);

    renderer.create_framebuffers(
        device,
        renderer.render_pass.expect("RenderPass was not initialized!"),
        swapchain.extent,
        &swapchain_image_views,
    );

    let vertices = [
        Vertex { pos: [0.0, -0.5, 0.0], padding: 0.0, color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [0.5, 0.5, 0.0], padding: 0.0, color: [0.0, 1.0, 0.0, 1.0] },
        Vertex { pos: [-0.5, 0.5, 0.0], padding: 0.0, color: [0.0, 0.0, 1.0, 1.0] },
    ];

    let vertex_buffer = AurenBuffer::new(
        &context.instance,
        optimal_gpu.handle,
        device,
        (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    vertex_buffer.upload_data(device, &vertices);

    let mut scene_data = SceneData::default();

    scene_data.model_matrix = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    scene_data.base_color = [1.0, 1.0, 1.0, 1.0];

    let scene_buffer = AurenBuffer::new(
        &context.instance,
        optimal_gpu.handle,
        device,
        std::mem::size_of::<SceneData>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    scene_buffer.upload_data(device, &[scene_data]);

    renderer.update_descriptor_sets(device.clone(), vertex_buffer.buffer, scene_buffer.buffer);

    let sync_objects = AurenRenderer::create_sync_objects(device);

    let command_pool_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(optimal_gpu.graphics_index.unwrap())
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

    let command_pool = unsafe {
        device.create_command_pool(&command_pool_info, None).unwrap()
    };

    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    let command_buffer = unsafe {
        device.allocate_command_buffers(&alloc_info).unwrap()[0]
    };

    let graphics_queue = unsafe {
        device.get_device_queue(optimal_gpu.graphics_index.unwrap(), 0)
    };

    let swapchain_images = swapchain.get_images(swapchain.swapchain_loader.clone(), swapchain.swapchain);
    let swapchain_image_views = swapchain.create_image_views(device, &swapchain_images);

    renderer.create_framebuffers(
        device,
        renderer.render_pass.expect("RenderPass must be created first"),
        swapchain.extent,
        &swapchain_image_views,
    );

    while !window_manager.get_window_by_id(0).expect("Err").window.should_close() {
        window_manager.update();
        
        unsafe {
            device.wait_for_fences(&[sync_objects.in_flight_fence], true, u64::MAX)
                .expect("Failed to wait for fence");

            device.reset_fences(&[sync_objects.in_flight_fence])
                .expect("Failed to reset fence");

            let (image_index, _is_suboptimal) = swapchain.swapchain_loader
                .acquire_next_image(
                    swapchain.swapchain,
                    u64::MAX,
                    sync_objects.image_available,
                    vk::Fence::null(),
                )
                .expect("Failed to acquire swapchain image");
            
            device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .expect("Failed to reset command buffer");

            renderer.record_commands(
                command_buffer,
                renderer.frame_buffers[image_index as usize],
                vertex_buffer.buffer,
                vertices.len() as u32,
            );

            let wait_semaphores = [sync_objects.image_available];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores = [sync_objects.render_finished];
            let command_buffers = [command_buffer];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            device.queue_submit(graphics_queue, &[submit_info], sync_objects.in_flight_fence)
                .expect("Failed to submit queue");

            let swapchains = [swapchain.swapchain];
            let image_indices = [image_index];

            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            swapchain.swapchain_loader
                .queue_present(graphics_queue, &present_info)
                .expect("Failed to present");
        }
    }

    unsafe {
        device.device_wait_idle().expect("Failed to wait for idle");
    }
}