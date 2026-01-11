# AurenFox API Structure

## File structure and assignments

| File          | Function                                   | Lifetime                     |
|---------------|--------------------------------------------|------------------------------|
| window.rs     | Owns the GLFW and the Surface              | Permanent                    |
| device.rs     | Picks the best GPU                         | Transient (used during init) |
| context.rs    | Owns the Instance and Logical Device       | Permanent                    |
| swapchain.rs  | Owns images and framebuffers               | Volatile (deleted on resize) |
| pipeline.rs   | Owns Shaders and Fixed-Function state      | Permanent                    |
| descriptor.rs | Manages Descriptor Pools and Layouts       | Permanent/Per-Frame          |
| buffer.rs     | Helper for allocating memory               | Utility                      |
| renderer.rs   | Records commands and Submits to Queue      | Per-Frame                    |
| types.rs      | Shared `#[repr(C)]` structs (Vertex/Scene) | Permanent                    |