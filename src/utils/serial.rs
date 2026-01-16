use serde::Deserialize;
use serialport::UsbPortInfo;

#[derive(Clone, Debug, PartialEq)]
pub struct SetVolumeProps {
    pub channel: u8,
    pub volume: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandsOut {
    RequestInfo,
    SetVolume(SetVolumeProps),
}

#[repr(u8)]
pub enum CommandOut {
    RequestInfo = 0x01,
    SetVolume = 0x02,
}

#[repr(u8)]
pub enum CommandIn {
    SendInfo = 0x81,
    SendVolume = 0x82,
}

impl TryFrom<u8> for CommandIn {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x81 => Ok(CommandIn::SendInfo),
            0x82 => Ok(CommandIn::SendVolume),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DeviceSliderData {
    pub name: String,
    pub set_volume_action: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DeviceInfo {
    pub sliders: Vec<DeviceSliderData>,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum CommandsIn {
    SendInfo(DeviceInfo),
    SendVolume(VolumeInfo),
}

#[derive(Clone, Debug, PartialEq)]
pub struct VolumeInfo {
    pub channel: u8,
    pub volume: u8,
}

pub fn find_serial_port(
    vid: u16,
    pid: u16,
) -> Result<(String, UsbPortInfo), Box<dyn std::error::Error>> {
    let ports = serialport::available_ports()?;

    for port in ports {
        if let serialport::SerialPortType::UsbPort(info) = &port.port_type {
            if dbg!(info).vid == vid && info.pid == pid {
                return Ok((port.port_name, info.clone()));
            }
        }
    }

    Err("Device not found".into())
}

pub fn get_payload(command: CommandsOut) -> Vec<u8> {
    let mut buffer = Vec::new();

    match command {
        CommandsOut::RequestInfo => {
            buffer.push(CommandOut::RequestInfo as u8);
            buffer.push(0x00); // channel (unused)
        }
        CommandsOut::SetVolume(props) => {
            buffer.push(CommandOut::SetVolume as u8);
            buffer.push(props.channel);
            buffer.push(props.volume);
        }
    }

    buffer
}

pub fn get_received_payload(buffer: &[u8]) -> Result<CommandsIn, Box<dyn std::error::Error>> {
    println!("Received buffer: {:#04X?}", buffer);
    if buffer.len() < 2 {
        return Err("Buffer too short".into());
    }
    let command = buffer[0];
    let channel = buffer[1];

    match CommandIn::try_from(command) {
        Ok(CommandIn::SendInfo) => {
            if buffer.len() < 3 {
                return Err("Buffer too short for SendInfo".into());
            }
            let json_raw = &buffer[2..];
            println!("{}", std::str::from_utf8(json_raw)?);
            Ok(CommandsIn::SendInfo(serde_json::from_slice::<DeviceInfo>(
                json_raw,
            )?))
        }
        Ok(CommandIn::SendVolume) => {
            let volume = buffer[2];
            Ok(CommandsIn::SendVolume(VolumeInfo {
                channel: channel,
                volume,
            }))
        }
        Err(_) => Err(format!("Unknown command received: {:#04X?}", command).into()),
    }
}

pub fn send_command(port: &mut Box<dyn serialport::SerialPort>, command: CommandsOut) {
    let mut buffer = Vec::new();

    match command {
        CommandsOut::RequestInfo => {
            buffer.push(CommandOut::RequestInfo as u8);
            buffer.push(0x00); // channel (unused)
        }
        CommandsOut::SetVolume(props) => {
            buffer.push(CommandOut::SetVolume as u8);
            buffer.push(props.channel + 1);
            buffer.push(props.volume);
        }
    }

    if let Err(e) = port.write_all(&buffer) {
        eprintln!("Failed to send command: {}", e);
    }
}
