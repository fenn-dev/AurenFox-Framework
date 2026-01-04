pub struct EngineSettings {
    // Graphics Features
    pub vsync: bool,
    pub use_data_pulling: bool,
    pub backface_culling: bool,
    
    // Window Settings
    pub initial_width: u32,
    pub initial_height: u32,
    pub title: &'static str,

    // Debugging
    pub enable_validation_layers: bool,
    pub log_level: LogLevel,
}

pub enum LogLevel {
    None,
    Minimal,
    Verbose,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            vsync: true,
            use_data_pulling: true, // You want this ON for your current shaders
            backface_culling: true,
            initial_width: 1280,
            initial_height: 720,
            title: "AurenFox Engine v2",
            enable_validation_layers: cfg!(debug_assertions), // Auto-enable in debug mode
            log_level: LogLevel::Verbose,
        }
    }
}