use crate::{
    view::{connection_window::ConnectionWindow, error_window::ErrorWindow},
    UiView,
};
use std::collections::HashMap;
use strum_macros::EnumIter;

#[derive(PartialEq, Hash, Eq, Clone, Copy, EnumIter)]
pub enum WindowType {
    ConnectionWindow,
    ErrorWindow,
}

pub struct WindowWrapper {
    window_map: HashMap<WindowType, Box<dyn UiView>>,
}

impl WindowWrapper {
    pub fn new() -> Self {
        let mut window_map = HashMap::<WindowType, Box<dyn UiView>>::new();
        window_map.insert(
            WindowType::ConnectionWindow,
            Box::new(ConnectionWindow::new()),
        );
        window_map.insert(WindowType::ErrorWindow, Box::new(ErrorWindow::new()));

        Self { window_map }
    }

    pub fn get_window(&mut self, window_type: WindowType) -> &mut Box<dyn UiView> {
        self.window_map.get_mut(&window_type).unwrap()
    }
}
