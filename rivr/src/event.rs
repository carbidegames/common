use {
    std::{
        cell::{Cell},
        rc::{Rc},
    }
};

#[derive(Clone)]
pub struct Event {
    sink: Rc<Cell<bool>>,
}

impl Event {
    pub fn new() -> Self {
        Event {
            sink: Rc::new(Cell::new(false))
        }
    }

    pub fn raise(&self) {
        self.sink.set(true)
    }

    pub fn check(&self) -> bool {
        self.sink.replace(false)
    }
}
