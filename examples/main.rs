use aurenfox::framework::AurenFoxFramework;
use aurenfox::glfwvulkan_agent::GLFWVulkanAgent;

fn program(_app: &AurenFoxFramework) {
    
}

fn main() {
    let mut app = AurenFoxFramework::new(GLFWVulkanAgent::new());

    let main_window_id = app.create_window("AurenFox Window", 800, 600, Some(0));

    app.assign_master(main_window_id.unwrap());


    let _ = app.create_window("Another Window", 1024, 768, Some(1));

    app.run(Box::new(|app_context| {
        program(app_context);
    }));
}