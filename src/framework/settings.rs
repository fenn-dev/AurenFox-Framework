pub enum Culling {
    Disable,
    Enable,
}

pub enum DrawOrder {
    Clockwise,
    CounterClockwise,
}

pub struct MeshBehaviour {
    pub culling: Culling,
    pub draw_order: DrawOrder,
}

impl MeshBehaviour {
    pub fn to_vulkan_cull_mode(&self) -> ash::vk::CullModeFlags {
        match self.culling {
            Culling::Enable => ash::vk::CullModeFlags::BACK,
            Culling::Disable => ash::vk::CullModeFlags::NONE,
        }
    }
}