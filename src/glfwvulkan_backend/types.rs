use ash::vk;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],    // float3
    pub padding: f32,     // padding (Alignment filler)
    pub color: [f32; 4],   // float4
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SceneData {
    pub model_matrix: [[f32; 4]; 4], // float4x4
    pub base_color: [f32; 4],    // float4
    pub time: f32,               // For math-based animations
    pub _padding: [f32; 3],      // Alignment to 16-byte boundary
}

// Default implementations to make initialization cleaner
impl Default for SceneData {
    fn default() -> Self {
        Self {
            model_matrix: [
                [1.0, 0.0, 0.0, 0.0], // Row 0
                [0.0, 1.0, 0.0, 0.0], // Row 1
                [0.0, 0.0, 1.0, 0.0], // Row 2
                [0.0, 0.0, 0.0, 1.0], // Row 3
            ],
            base_color: [1.0, 1.0, 1.0, 1.0],
            time: 0.0,
            _padding: [0.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupportDetails {
    pub fn new(physical_device: vk::PhysicalDevice, surface_loader: &ash::khr::surface::Instance, surface: vk::SurfaceKHR) -> Self {
        unsafe {
            let capabilities = surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .expect("Failed to get surface capabilities");

            let formats = surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .expect("Failed to get surface formats");

            let present_modes = surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .expect("Failed to get surface present modes");

            Self {
                capabilities,
                formats,
                present_modes,
            }
        }
    }
}