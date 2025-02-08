use serial_enumerator::get_serial_list;

pub struct Settings;

impl Settings {
    pub fn get_port_names() -> Vec<String> {
        let port_info = get_serial_list();
        port_info.iter().map(|x| x.name.clone()).collect()
    }
}
