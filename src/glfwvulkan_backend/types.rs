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
    pub model_matrix: [f32; 16], // float4x4
    pub base_color: [f32; 4],    // float4
    pub time: f32,               // For math-based animations
    pub _padding: [f32; 3],      // Alignment to 16-byte boundary
}

// Default implementations to make initialization cleaner
impl Default for SceneData {
    fn default() -> Self {
        Self {
            model_matrix: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
            base_color: [1.0, 1.0, 1.0, 1.0],
            time: 0.0,
            _padding: [0.0; 3],
        }
    }
}

pub struct swapchain_support_details {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl swapchain_support_details {
    fn is_complete(&self) -> bool {
        return !self.formats.is_empty() && !self.present_modes.is_empty();
    }
}