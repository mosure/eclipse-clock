use std::time::SystemTime;


pub struct AppContext {
    pub width: u32,
    pub height: u32,
    pub boot_time: SystemTime,
}

pub trait Stage {
    fn render(&mut self) -> ();
}
