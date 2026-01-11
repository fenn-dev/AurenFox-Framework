# AurenFox Framework (Rust) Setup

This guide is to help you setup the AurenFox Framework so that it may be used to create the app of your choice!

## Setting up modules

1. ) First step is to use the propper modules. It is recommended to write `use aurenfox::framework::AurenFoxFramework;` at the top of the file as this is the main framework

2. ) Based on what backend you want to use, install the appropriate backend-agent. For this example, we will do `use aurenfox::glfwvulkan_agent::GLFWVulkanAgent;` as glfw and Vulkan is a great combination for high performant cross platform applications.

## Creating the app engine

1. ) You'll want to create the app within your main function. For this, define a `let mut app` variable. And assign it (`=`) to `AurenFoxFramework::new(_AGENT HERE_);`

2. ) We want to assign an agent to the parameter of the AurenFoxFramework. This is to assign the backend. Since we've imported the GLFWVulkan Agent. We can simply write `GLFWVulkanAgent::new()` into the parameter field.

A backend of your own can be created and must follow the `interfaces::RHI` struct in order to work with the AurenFox Framework.

## Running the AurenFox Framework

This can be done by simply calling the run function. Based on the example. We would do `app.run(None);`

to run code within the loop. All that needs to be done, is to simply define a function with the parameters: `app: &AurenFoxFramework` and no return type.
Once this is done, simply insert the function into the run function of the framework. For example `app.run(Some(Box::new(|app_context| {
        program(app_context);
    })));`

However, you may notice that the program immediatly terminates upon launch. This is due to the lack of windows being present.

To create windows, please see the [windows](windows.md) documentation

## Function Signatures

Initializing the framework: `aurenfox::framework::AurenFoxFramework
pub fn new<T>(backend_struct: T) -> Self
where
    T: RHI + 'static,`

Running the framework: `fn run(&mut self, mut user_code: Option<Box<dyn FnMut(&mut AurenFoxFramework) + 'static, Global>>)`
