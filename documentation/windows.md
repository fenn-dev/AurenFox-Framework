# AurenFox Window

## Basic Signatures

**Window creation**:
`AurenFoxFramework::create_window(&mut self, title, width, height, id)`

Creates a window with a title, size and id.

id is an optional type and if defined as `None`, it will auto increment to the first unused slot. The first window will always be id 0.

To manually assign an id. Simply insert Some(n) into the id parameter.

The program will not close until all windows are closed. Closing a window, also kills it from memory.

Whilst creating a window. One can also assign it to a let variable as to store the ID of it. This is especially useful if the ID is automatically generated.

-----

**Asigning a master window**: `AurenFoxFramework::assign_master(&mut self, id);`

passing in the id to the assign_master function assigns a master window.

If the master window closes, all other windows will terminate as well and the same with the rest of the program, deeming its end of execution.

New masters can be assigned, but must be done before the start of the frame.

Assigning a master window is completly optional, but often recommended if your program is based on one main window.

-----

**Terminating windows**:
`AurenFoxFramework::queue_destroy(&self, id);`

Based on the passed id. The window will be queued for termination. This is especially useful if your window needs to close early or automatically

## technical info

Windows get automatically cleared up at the start of each frame, so does checking if the master window is assigned and terminated or if all the windows are closed.

If the master window is terminated, or all windows are closed. The program terminates.

## Function Signatures

Creating a window: `aurenfox::framework::AurenFoxFramework
pub fn create_window(&mut self, title: &str, width: u32, height: u32, id: Option<usize>) -> Result<usize, String>`

Assigning a master window: `aurenfox::framework::AurenFoxFramework
pub fn assign_master(&mut self, id: usize)`
