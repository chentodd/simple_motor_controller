use crate::{
    view::{
        command_window::CommandWindow, connection_window::ConnectionWindow,
        control_mode_window::ControlModeWindow, error_window::ErrorWindow,
        profile_window::DataGraph,
    },
    UiView, DEFAULT_GRAPH_SIZE,
};
use std::collections::HashMap;
use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy, EnumIter)]
pub enum WindowType {
    ConnectionWindow,
    ControlModeWindow,
    CommandWindow,
    ProfileWindow,
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
        window_map.insert(
            WindowType::ControlModeWindow,
            Box::new(ControlModeWindow::new()),
        );
        window_map.insert(WindowType::CommandWindow, Box::new(CommandWindow::new()));
        window_map.insert(
            WindowType::ProfileWindow,
            Box::new(DataGraph::new(DEFAULT_GRAPH_SIZE)),
        );
        window_map.insert(WindowType::ErrorWindow, Box::new(ErrorWindow::new()));

        Self { window_map }
    }

    pub fn get_window(&mut self, window_type: WindowType) -> &mut Box<dyn UiView> {
        self.window_map.get_mut(&window_type).unwrap()
    }
}
