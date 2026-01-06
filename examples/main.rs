use std::ffi::CString;
use aurenfox::glfwvulkan_backend::{
    context::AurenContext,
    device::AurenDevice,
    window::AurenWindowManager,
    swapchain::AurenSwapChain,
    types::SwapchainSupportDetails,
    renderer::AurenRenderer,
};

fn main() {
    

    let mut window_manager = AurenWindowManager::new();

    let glfw_extensions = window_manager.glfw.get_required_instance_extensions()
    .expect("Failed to get required instance extensions");

    // 2. Convert to CStrings and KEEP THEM ALIVE in a variable
    let c_extensions: Vec<CString> = glfw_extensions
        .iter()
        .map(|ext| CString::new(ext.as_str()).expect("Failed to convert extension name"))
        .collect();

    // 3. Create the pointers from the CStrings
    let extension_ptrs: Vec<*const i8> = c_extensions
        .iter()
        .map(|c_ext| c_ext.as_ptr())
        .collect();

    // 4. Pass the pointers. (c_extensions must exist until this call finishes)
    let mut context = AurenContext::new(
        &"AurenFox".to_string(), 
        0x01000000, 
        &"AurenFox Engine".to_string(), 
        0x01000000, 
        &extension_ptrs
    );

    let device_manager = AurenDevice::new(&context.instance);

    context.create_logical_device(device_manager.devices[0].handle);

    device_manager.print_all_devices();

    window_manager.create_window(&context.entry, &context.instance,"AurenFox Window", 720, 480, Some(0)).expect("It couldnt create a window. L");

    let details = SwapchainSupportDetails::new(device_manager.get_optimal_device().expect("Couldnt find an optimal device with vulkan support").handle, &context.surface_loader, window_manager.get_window_by_id(0).expect("Could not find any windows").surface);

    let window_entry = window_manager.get_window_by_id(0).expect("Err");

    let window_ref = &window_entry.window;
    let surface_ref = &window_entry.surface;

    let swapchain = AurenSwapChain::new(
        &context.instance, 
        context.logical_device.as_ref().expect("err"), 
        window_ref,
        surface_ref,
        true,
        &details
    );

    let mut renderer = AurenRenderer::new(context.logical_device.clone().expect("err"), swapchain.extent, swapchain.image_format.format);

    renderer.create_render_pass(swapchain.image_format.format, &context.logical_device.clone().expect("err"));

    

    while !window_manager.get_window_by_id(0).expect("Couldnt find window").window.should_close() {
        window_manager.update();
    }
}