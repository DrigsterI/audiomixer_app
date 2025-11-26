pub fn find_device(vid: u16, pid: u16) -> Option<String> {
    match serialport::available_ports() {
        Ok(ports) => {
            for port in ports {
                match &port.port_type {
                    serialport::SerialPortType::UsbPort(info) => {
                        if info.vid == vid && info.pid == pid {
                            return Some(port.port_name);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        Err(_) => None,
    }
}
