use std::u64;
use ash::vk;
use aurenfox::{
    glfwvulkan_backend::{
        buffers::AurenBuffer, context::AurenContext, device::AurenDevice, 
        helpers, renderer::AurenRenderer, types::SwapchainSupportDetails, 
        window::AurenWindowManager
    }, 
    types::{SceneData, Vertex}
};

fn main() {
    helpers::log_info("Main", "Initializing AurenFox Engine");

    // --- 1. System Setup ---
    let mut window_manager = AurenWindowManager::new();
    let required_extensions = window_manager.glfw.get_required_instance_extensions()
        .expect("GLFW: Failed to get required instance extensions");

    // Extension pointers must stay alive for Context creation
    let extension_names_c: Vec<std::ffi::CString> = required_extensions
        .iter()
        .map(|ext| std::ffi::CString::new(ext.as_str()).unwrap())
        .collect();
    let extension_ptrs: Vec<*const i8> = extension_names_c.iter().map(|c| c.as_ptr()).collect();

    let mut context = AurenContext::new(
        &"AurenFox App".to_string(), 1,
        &"AurenFox Engine".to_string(), 1,
        &extension_ptrs
    );

    // --- 2. Window Creation ---
    // We create the window early so we can use its surface to pick the right GPU
    window_manager.create_window(&context.entry, &context.instance, "AurenFox Triangle", 1280, 720, Some(0))
        .expect("Failed to create window");
    
    let window_entry = window_manager.get_window_by_id(0).expect("Err: Window 0 missing");

    // --- 3. Device & Renderer Setup ---
    let device_manager = AurenDevice::new(&context.instance);
    let optimal_gpu = device_manager.get_optimal_device().expect("No compatible GPU found");
    context.create_logical_device(optimal_gpu.handle);
    
    let device = context.logical_device.as_ref().expect("Logical device missing").clone();
    let graphics_queue = unsafe { device.get_device_queue(optimal_gpu.graphics_index.unwrap(), 0) };

    // Initialize the Renderer (Stateless global resources like Pipelines)
    let mut renderer = AurenRenderer::new(device.clone(), vk::Extent2D { width: 1280, height: 720 });

    // --- 4. Pipeline & RenderPass Setup ---
    // Note: Use a temporary swapchain detail just to get the format for the RenderPass
    let temp_details = SwapchainSupportDetails::new(optimal_gpu.handle, &context.surface_loader, window_entry.surface);
    renderer.create_render_pass(temp_details.formats[0].format, &device);

    let v_mod = renderer.create_shader_module(std::fs::read("src/shaders/vert.spv").unwrap());
    let f_mod = renderer.create_shader_module(std::fs::read("src/shaders/frag.spv").unwrap());
    renderer.create_graphics_pipeline(v_mod, f_mod);

    // --- 5. Prepare Window Resources ---
    // This attaches the Swapchain, CommandPool, and SyncObjects to the window itself
    renderer.prepare_window(
        optimal_gpu.graphics_index.unwrap(),
        &context.instance,
        window_entry,
        true, // vsync
        &temp_details
    );

    // --- 6. Data Buffers ---
    let vertices = [
        Vertex { pos: [0.0, -0.5, 0.0], padding: 0.0, color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { pos: [0.5, 0.5, 0.0], padding: 0.0, color: [0.0, 1.0, 0.0, 1.0] },
        Vertex { pos: [-0.5, 0.5, 0.0], padding: 0.0, color: [0.0, 0.0, 1.0, 1.0] },
    ];

    let vertex_buffer = AurenBuffer::new(
        &context.instance, optimal_gpu.handle, &device,
        (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    vertex_buffer.upload_data(&device, &vertices);

    let scene_buffer = AurenBuffer::new(
        &context.instance, optimal_gpu.handle, &device,
        std::mem::size_of::<SceneData>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    scene_buffer.upload_data(&device, &[SceneData::default()]);

    // Bind data to the renderer's descriptor sets
    renderer.update_descriptor_sets(vertex_buffer.buffer, scene_buffer.buffer);

    // --- 7. Main Loop ---
    let swapchain_loader = ash::khr::swapchain::Device::new(&context.instance, &device);

    while !window_manager.get_window_by_id(0).unwrap().window.should_close() {
        window_manager.glfw.wait_events_timeout(0.001);
        window_manager.update();

        let window = window_manager.get_window_by_id(0).unwrap();

        // Handle Resize
        if window.take_resized() {
            let (w, h) = window.window.get_framebuffer_size();
            if w == 0 || h == 0 {
                println!("Window minimized");
                continue; 
            }
            renderer.extent = vk::Extent2D { width: w as u32, height: h as u32 };

            unsafe { device.device_wait_idle().unwrap(); }
            let updated_details = SwapchainSupportDetails::new(optimal_gpu.handle, &context.surface_loader, window.surface);
            
            println!("Redoing the extent");
            renderer.extent = vk::Extent2D { width: w as u32, height: h as u32 };

            println!("Preparing window");
            renderer.prepare_window(
                optimal_gpu.graphics_index.unwrap(),
                &context.instance, window, true, &updated_details
            );

            renderer.render_to(
                window,
                &swapchain_loader,
                graphics_queue,
                vertices.len() as u32
            );
            continue;
        }

        renderer.render_to(
            window,
            &swapchain_loader,
            graphics_queue,
            vertices.len() as u32
        );
    }

    unsafe {
        device.device_wait_idle().expect("Failed to wait for idle");
    }
}