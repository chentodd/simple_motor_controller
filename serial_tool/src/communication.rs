use serial_enumerator::{get_serial_list, SerialInfo};

#[derive(Default)]
pub struct ConnectionSettings {
    serial_ports: Vec<SerialInfo>,
}

impl ConnectionSettings {
    pub fn new() -> Self {
        Self {
            serial_ports: get_serial_list(),
            ..Default::default()
        }
    }

    pub fn get_port_names(&self) -> Vec<&str> {
        self.serial_ports.iter().map(|x| x.name.as_str()).collect()
    }
}
