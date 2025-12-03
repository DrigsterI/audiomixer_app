use std::time::Duration;

use crate::{
    pages::{Loading, Main},
    utils::{Commands, find_serial_port},
};
use async_broadcast::broadcast;
use async_std::task::sleep;
use freya::prelude::*;
use freya_radio::prelude::*;
use freya_router::prelude::*;
use serialport::{SerialPort, UsbPortInfo};

#[derive(Default)]
#[allow(dead_code)]
pub struct Data {
    pub serial_port: Option<Box<dyn SerialPort>>,
    pub device_info: Option<UsbPortInfo>,
    pub broadcast_tx: Option<async_broadcast::Sender<Commands>>,
}

#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
#[allow(dead_code)]
pub enum DataChannel {
    DeviceUpdate,
    NoUpdate,
}

impl RadioChannel<Data> for DataChannel {}

const TARGET_VID: u16 = 0x303a;
const TARGET_PID: u16 = 0x8145;
#[allow(non_snake_case)]
pub fn App() -> impl IntoElement {
    use_init_radio_station::<Data, DataChannel>(Data::default);

    let mut radio = use_radio::<Data, DataChannel>(DataChannel::DeviceUpdate);
    
    // Hook 1: Setup broadcast channel and find device
    use_hook(|| {
        spawn(async move {
            let (tx, _rx) = broadcast::<Commands>(2);

            radio
                .write()
                .broadcast_tx = Some(tx);

            loop {
                match find_serial_port(TARGET_VID, TARGET_PID) {
                    Ok(port_info) => {
                        println!("Device found! {}", port_info.0);
                        let mut port = serialport::new(&port_info.0, 115_200)
                            .timeout(Duration::from_millis(100))
                            .dtr_on_open(true)
                            .open()
                            .expect("Failed to open port");
                        
                        // Send initial commands to get device info and volume states
                        println!("Requesting device info...");
                        crate::utils::send_command(&mut port, Commands::RequestInfo);
                        sleep(Duration::from_millis(50)).await;
                        
                        println!("Requesting volume states...");
                        crate::utils::send_command(&mut port, Commands::RequestVolume);
                        sleep(Duration::from_millis(50)).await;
                        
                        radio
                            .write()
                            .serial_port = Some(port);
                        radio
                            .write()
                            .device_info = Some(port_info.1);
                        break;
                    }
                    Err(e) => {
                        println!("Device not found! {}", e);
                    }
                };
                sleep(Duration::from_secs(2)).await;
            }
        });
    });

    // Hook 2: Command sending loop
    let mut radio_cmd = use_radio::<Data, DataChannel>(DataChannel::DeviceUpdate);
    use_hook(|| {
        spawn(async move {
            // Wait for broadcast channel to be initialized
            let mut rx = loop {
                if let Some(tx) = &radio_cmd.read().broadcast_tx {
                    break tx.new_receiver();
                }
                sleep(Duration::from_millis(10)).await;
            };
            
            loop {
                match rx
                    .recv()
                    .await
                {
                    Ok(command) => {
                        println!("Received command: {:?}", command);
                        if let Some(port) = &mut radio_cmd.write().serial_port {
                            crate::utils::send_command(port, command);
                        }
                    },
                    Err(_) => continue, // Channel closed
                }
            }
        });
    });

    // Hook 2: Read from serial port
    let mut radio_read = use_radio::<Data, DataChannel>(DataChannel::DeviceUpdate);
    use_hook(|| {
        spawn(async move {
            let mut buffer = vec![0; 256];
            loop {
                if let Some(port) = &mut radio_read.write().serial_port {
                    match port.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            let data = String::from_utf8_lossy(&buffer[..n]);
                            println!("Serial RX: {}", data.trim());
                        }
                        Ok(_) => {
                            // Timeout - no data
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                            // Expected timeout, keep looping
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                            // Device disconnected, reset state
                            radio_read.write().serial_port = None;
                            radio_read.write().device_info = None;
                            continue;
                        }
                        Err(e) => {
                            eprintln!("Serial read error: {}", e);
                        }
                    }
                }
                sleep(Duration::from_millis(10)).await;
            }
        });
    });

    router::<Route>(|| RouterConfig::default().with_initial_path(Route::Main))
}

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Loading,
    #[route("/main")]
    Main,
}
