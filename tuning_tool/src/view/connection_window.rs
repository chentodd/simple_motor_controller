use std::fmt::Display;

use eframe::egui::{Button, ComboBox, Ui};
use nusb;

use crate::{UiView, ViewEvent, ViewRequest};

#[derive(Default, PartialEq)]
struct UsbInfo {
    vendor_id: u16,
    product_id: u16,
    product_string: String,
}

impl Display for UsbInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:#06x}, {:#06x}, {}",
            self.vendor_id, self.product_id, self.product_string
        )
    }
}

impl From<nusb::DeviceInfo> for UsbInfo {
    fn from(device: nusb::DeviceInfo) -> Self {
        Self {
            vendor_id: device.vendor_id(),
            product_id: device.product_id(),
            product_string: device.product_string().unwrap_or_default().to_string(),
        }
    }
}

#[derive(Default)]
pub(super) struct ConnectionWindow {
    selected_usb: UsbInfo,
    target: bool,
    curr: bool,
    request: Option<ViewRequest>,
}

impl ConnectionWindow {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl UiView for ConnectionWindow {
    fn show(&mut self, ui: &mut Ui) {
        let devices = nusb::list_devices().unwrap();

        ui.heading("Connection setup");
        ui.horizontal_centered(|ui| {
            ComboBox::new("usb", "usbs")
                .selected_text(format!("{}", self.selected_usb))
                .show_ui(ui, |ui| {
                    for device in devices {
                        let usb_info = UsbInfo::from(device);
                        let text = usb_info.to_string();
                        ui.selectable_value(&mut self.selected_usb, usb_info, text);
                    }
                });

            let text_in_button = if self.curr { "Stop" } else { "Start" };
            let conn_button = Button::new(text_in_button);

            if ui
                .add_enabled(!self.selected_usb.product_string.is_empty(), conn_button)
                .clicked()
            {
                self.target = !self.curr;
                if self.target {
                    self.request = Some(ViewRequest::ConnectionStart(
                        self.selected_usb.product_string.clone(),
                    ));
                } else {
                    self.request = Some(ViewRequest::ConnectionStop);
                }
            }
        });
    }

    fn take_request(&mut self) -> Option<ViewRequest> {
        self.request.take()
    }

    fn handle_event(&mut self, event: ViewEvent) {
        match event {
            ViewEvent::ConnectionStatusUpdate(x) => self.curr = x,
            _ => (),
        }
    }

    fn reset(&mut self) {
        // Start or stop fails, reset flags, Ex: if user encounters stop failure,
        // 1. target = true
        // 2. curr = false
        //
        // target will be set to curr when error occurred, so user can try to
        // start again
        self.target = self.curr
    }
}
