use ash::khr::{self, swapchain};
use aurenfox::glfwvulkan_backend::{context::AurenContext, device::AurenDevice, window::AurenWindowManager, swapchain::AurenSwapChain};

fn main() {
    

    let context = AurenContext::new(&"AurenFox".to_string(), 0x01000000, &"AurenFox Engine".to_string(), 0x01000000, &[khr::swapchain::NAME.as_ptr()]);

    let device_manager = AurenDevice::new(&context.instance);

    context.create_logical_device(device_manager.devices[0].handle);

    let mut window_manager = AurenWindowManager::new();

    device_manager.print_all_devices();

    window_manager.create_window("AurenFox Window", 720, 480, Some(0)).expect("It couldnt create a window. L");

    let swapchain = AurenSwapChain::new(&context.instance, Some(context.logical_device), window_manager.get_window_by_id(0), graphics.surface, true, details)

    while window_manager.get_window_by_id(0).is_some() {
        window_manager.update();
    }
}