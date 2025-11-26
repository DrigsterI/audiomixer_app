use std::error::Error;

use rusb::{Context, Device, UsbContext};

pub fn find_device(vid: u16, pid: u16) -> Option<Device<Context>> {
    match Context::new() {
        Ok(context) => match context.devices() {
            Ok(devices) => {
                for device in devices.iter() {
                    if let Ok(desc) = device.device_descriptor() {
                        if desc.vendor_id() == vid && desc.product_id() == pid {
                            println!("Device found!");
                            return Some(device);
                        }
                    }
                }
                None
            }
            Err(e) => {
                println!("Failed to get device list");
                None
            }
        },
        Err(e) => {
            println!("Failed to get device context");
            None
        }
    }
}
