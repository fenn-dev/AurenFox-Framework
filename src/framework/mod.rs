use crate::interfaces::{RHI};
use std::{cell::RefCell};
pub struct AurenFoxFramework {
    pub backend: Box<dyn RHI>,
    pub destroy_queue: std::cell::RefCell<Vec<usize>>,
}

impl AurenFoxFramework {
    pub fn new<T: RHI + 'static>(backend_struct: T) -> Self {
        Self {
            backend: Box::new(backend_struct),
            destroy_queue: RefCell::new(Vec::new()),
        }
    }

    pub fn create_window(&mut self, title: &str, width: u32, height: u32, id: Option<usize>) -> Result<usize, String> {
        return self.backend.create_window(title, width, height, id);
    }

    pub fn run(&mut self, mut user_code: Option<Box<dyn FnMut(&mut AurenFoxFramework) + 'static>>) {
        while !self.backend.should_close() {
            self.process_destroy_queue();
            self.backend.start_frame();

            if let Some(ref mut code) = user_code {
                code(self);
            }
            
            self.backend.end_frame();
        }
    }

    pub fn queue_destroy(&self, id: usize) {
        self.destroy_queue.borrow_mut().push(id);
    }

    pub fn assign_master(&mut self, id: usize) {
        self.backend.assign_master(id);
    }

    fn process_destroy_queue(&mut self) {
        let targets: Vec<usize> = self.destroy_queue.borrow_mut().drain(..).collect();
        for id in targets {
            self.destroy(&id);
        }
    }

    fn destroy(&mut self, id: &usize) {
        self.backend.destroy_window(*id);
    }

    
}

