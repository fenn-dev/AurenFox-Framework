// Render Hardware Interface
pub trait RHI {
    fn create_window(&mut self, title: &str, width: u32, height: u32, id: Option<usize>) -> Result<usize, String>;

    fn new(&mut self);

    fn destroy_window(&mut self, id: usize);

    fn start_frame(&mut self);

    fn end_frame(&mut self);

    fn should_close(&self) -> bool;

    fn assign_master(&mut self, id: usize);
}