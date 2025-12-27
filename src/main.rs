use std::{
    ops::Index,
    sync::{Arc, Mutex},
    thread::{self},
    time::Duration,
};

use freya::prelude::*;
use freya_radio::hooks::{RadioChannel, RadioStation};
use futures_channel::mpsc::UnboundedSender;
use serialport::{SerialPort, UsbPortInfo};
use smol::Timer;

use crate::{
    app::App,
    utils::{CommandsIn, CommandsOut, find_serial_port},
};

mod app;
mod components;
mod pages;
mod utils;

#[derive(Default)]
#[allow(dead_code)]
pub struct DeviceInfo {
    pub usb_info: Option<UsbPortInfo>,
    pub slider_count: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderData {
    pub name: String,
    pub volume: u8,
    pub set_volume_action: String,
}

#[derive(Default)]
#[allow(dead_code)]
pub struct Data {
    pub device_info: DeviceInfo,
    pub serial_out_tx: Option<UnboundedSender<CommandsOut>>,
    pub sliders: Vec<SliderData>,
}

#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
#[allow(dead_code)]
pub enum DataChannel {
    DeviceInfoUpdate,
    SerialOutRxUpdate,
    SlidersUpdate,
    NoUpdate,
}

pub enum ChannelSend {
    DeviceInfoUpdate(Option<UsbPortInfo>),
    SliderVolumeUpdate(u8, u8),
    SliderInfoUpdate(u8, SliderData),
}

impl RadioChannel<Data> for DataChannel {}

const TARGET_VID: u16 = 0x303a;
const TARGET_PID: u16 = 0x8145;

fn main() {
    let mut radio_station: RadioStation<Data, DataChannel> =
        RadioStation::create_global(Data::default());
    let serial_port: Arc<Mutex<Option<Box<dyn SerialPort>>>> = Arc::new(Mutex::new(None));

    let (state_tx, mut state_rx) = futures_channel::mpsc::unbounded::<ChannelSend>();
    let (serial_out_tx, mut serial_out_rx) = futures_channel::mpsc::unbounded::<CommandsOut>();

    radio_station
        .write_channel(DataChannel::SerialOutRxUpdate)
        .serial_out_tx = Some(serial_out_tx.clone());

    // Thread to scan and connect to the device
    let serial_port_clone = serial_port.clone();
    let state_tx_clone = state_tx.clone();
    let serial_out_tx_clone = serial_out_tx.clone();
    thread::spawn(move || {
        smol::block_on(async {
            println!("Starting device scan...");
            loop {
                println!("Scanning for device...");
                match find_serial_port(TARGET_VID, TARGET_PID) {
                    Ok(port_info) => {
                        println!("Device found! {}", port_info.0);
                        let port = serialport::new(&port_info.0, 115_200)
                            .timeout(Duration::from_millis(10))
                            .dtr_on_open(true)
                            .open()
                            .expect("Failed to open port");

                        // Send initial commands to get device info and volume states
                        println!("Requesting device info...");

                        let _ = serial_out_tx_clone.unbounded_send(CommandsOut::RequestInfo);

                        serial_port_clone
                            .lock()
                            .unwrap()
                            .replace(port);

                        let _ = state_tx_clone
                            .unbounded_send(ChannelSend::DeviceInfoUpdate(Some(port_info.1)));
                        break;
                    }
                    Err(e) => {
                        println!("Device not found! {}", e);
                    }
                };
                Timer::after(Duration::from_secs(2)).await;
            }
        });
    });

    // Sender thread
    let serial_port_clone_2 = serial_port.clone();
    thread::spawn(move || {
        smol::block_on(async {
            loop {
                match serial_out_rx.try_next() {
                    Ok(Some(command)) => {
                        if let Some(port) = &mut serial_port_clone_2
                            .lock()
                            .unwrap()
                            .as_mut()
                        {
                            println!("Sending command: {:?}", command);
                            port.write_all(&crate::utils::get_payload(command))
                                .unwrap();
                        }
                    }
                    _ => continue,
                }
            }
        });
    });

    // Receiver thread
    let serial_port_clone = serial_port.clone();
    thread::spawn(move || {
        smol::block_on(async {
            let mut read_buffer: Vec<u8> = vec![0; 1024];
            let mut pending_buffer: Vec<u8> = Vec::new();
            let mut last_data_time = std::time::Instant::now();

            loop {
                if let Some(port) = &mut serial_port_clone
                    .lock()
                    .unwrap()
                    .as_mut()
                {
                    match port.read(&mut read_buffer) {
                        Ok(n) if n > 0 => {
                            let received = &read_buffer[..n];
                            println!("Received command buffer: {:#04X?}", received);

                            // Append new data to pending buffer
                            pending_buffer.extend_from_slice(received);
                            last_data_time = std::time::Instant::now();
                        }
                        Ok(_) | Err(_)
                            if !pending_buffer.is_empty()
                                && last_data_time.elapsed() > Duration::from_millis(100) =>
                        {
                            // Timeout with pending data - process what we have
                            eprintln!(
                                "Warning: timeout with {} bytes pending, clearing buffer",
                                pending_buffer.len()
                            );
                            pending_buffer.clear();
                        }
                        Ok(_) => {
                            // Timeout - no data
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                            // Expected timeout, keep looping
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                            // Device disconnected, reset state
                            let _ = state_tx.unbounded_send(ChannelSend::DeviceInfoUpdate(None));
                            continue;
                        }
                        Err(e) => {
                            eprintln!("Serial read error: {}", e);
                        }
                    }
                }

                // Process complete packets (terminated by 0xFF)
                while let Some(terminator_pos) = pending_buffer
                    .iter()
                    .position(|&b| b == 0xFF)
                {
                    let packet: Vec<u8> = pending_buffer
                        .drain(..terminator_pos)
                        .collect();
                    pending_buffer.drain(..1); // Remove the 0xFF terminator

                    if !packet.is_empty() {
                        match crate::utils::get_received_payload(&packet) {
                            Ok(command) => {
                                println!("Received command: {:?}", command);
                                match command {
                                    CommandsIn::SendInfo(device_info) => {
                                        println!("Received device info: {:?}", device_info);
                                        for (i, slider) in device_info
                                            .sliders
                                            .iter()
                                            .enumerate()
                                        {
                                            let _ = state_tx.unbounded_send(
                                                ChannelSend::SliderInfoUpdate(
                                                    (i + 1) as u8,
                                                    SliderData {
                                                        name: slider
                                                            .name
                                                            .clone(),
                                                        volume: 0,
                                                        set_volume_action: slider
                                                            .set_volume_action
                                                            .clone(),
                                                    },
                                                ),
                                            );
                                        }
                                    }
                                    CommandsIn::SendVolume(volume_info) => {
                                        println!("Received volume info: {:?}", volume_info);
                                        let _ = state_tx.unbounded_send(
                                            ChannelSend::SliderVolumeUpdate(
                                                volume_info.channel,
                                                volume_info.volume,
                                            ),
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse incoming payload: {}", e);
                            }
                        }
                    }
                }

                Timer::after(Duration::from_millis(1)).await;
            }
        });
    });

    launch(
        LaunchConfig::new()
            .with_future(async move {
                use futures::StreamExt;
                // State thread
                while let Some(state_update) = state_rx
                    .next()
                    .await
                {
                    match state_update {
                        ChannelSend::DeviceInfoUpdate(usb_port_info) => {
                            radio_station
                                .write_channel(DataChannel::DeviceInfoUpdate)
                                .device_info
                                .usb_info = usb_port_info;
                        }
                        ChannelSend::SliderVolumeUpdate(channel, volume) => {
                            let sliders = &mut radio_station
                                .write_channel(DataChannel::SlidersUpdate)
                                .sliders;
                            let index = channel as usize - 1;
                            if sliders.len() > index {
                                sliders[index].volume = volume;
                            }
                        }
                        ChannelSend::SliderInfoUpdate(channel, slider) => {
                            println!("Slider info update: {:?}", slider);
                            let sliders = &mut radio_station
                                .write_channel(DataChannel::SlidersUpdate)
                                .sliders;
                            let index = channel as usize - 1;
                            while sliders.len() <= index {
                                sliders.push(SliderData {
                                    name: "".to_string(),
                                    volume: 0,
                                    set_volume_action: "".to_string(),
                                });
                            }
                            sliders[index] = slider;
                        }
                    }
                }
            })
            .with_window(
                WindowConfig::new(FpRender::from_render(App {
                    radio_station,
                    serial_out_tx,
                }))
                .with_size(1200.0, 800.0), // .with_title("Audiomixer"),
            ),
    )
}
