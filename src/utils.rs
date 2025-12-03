use serialport::UsbPortInfo;

#[derive(Clone, Debug, PartialEq)]
pub struct SetVolumeProps {
    pub channel: u8,
    pub volume: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Commands {
    RequestInfo,
    RequestVolume,
    SetVolume(SetVolumeProps),
}

#[repr(u8)]
pub enum Command {
    RequestInfo = 0x01,
    RequestVolume = 0x02,
    SetVolume = 0x03,
}

pub fn find_serial_port(
    vid: u16,
    pid: u16,
) -> Result<(String, UsbPortInfo), Box<dyn std::error::Error>> {
    let ports = serialport::available_ports()?;

    for port in ports {
        if let serialport::SerialPortType::UsbPort(info) = &port.port_type {
            if info.vid == vid && info.pid == pid {
                return Ok((port.port_name, info.clone()));
            }
        }
    }

    Err("Device not found".into())
}

pub fn send_command(port: &mut Box<dyn serialport::SerialPort>, command: Commands) {
    let mut buffer = Vec::new();

    match command {
        Commands::RequestInfo => {
            buffer.push(Command::RequestInfo as u8);
            buffer.push(0x00); // channel (unused)
        }
        Commands::RequestVolume => {
            buffer.push(Command::RequestVolume as u8);
            buffer.push(0x00); // channel (unused)
        }
        Commands::SetVolume(props) => {
            buffer.push(Command::SetVolume as u8);
            buffer.push(props.channel + 1);
            buffer.push(props.volume);
        }
    }

    if let Err(e) = port.write_all(&buffer) {
        eprintln!("Failed to send command: {}", e);
    }
}
