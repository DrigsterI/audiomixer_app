#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use freya::{
    prelude::*,
    radio::{RadioChannel, RadioStation},
    tray::{
        TrayEvent, TrayIconBuilder,
        menu::{Menu, MenuEvent, MenuItem},
    },
};

use futures_channel::mpsc::UnboundedSender;
use futures_lite::StreamExt;
use serialport::{SerialPort, UsbPortInfo};
use smol::Timer;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use utils::send_command;

mod app;
mod components;
mod pages;
mod utils;

use app::App;

use crate::utils::{CommandsIn, CommandsOut, find_serial_port, run_action};

const ICON: &[u8] = include_bytes!("./freya_icon.png");
const TARGET_VID: u16 = 0x303a;
const TARGET_PID: u16 = 0x8145;

fn main() {
    let mut radio_station = RadioStation::create_global(Data::default());

    let tray_icon = || {
        let tray_menu = Menu::new();
        let _ = tray_menu.append(&MenuItem::new("Open", true, None));
        let _ = tray_menu.append(&MenuItem::new("Add slider", true, None));
        let _ = tray_menu.append(&MenuItem::new("Exit", true, None));
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Freya Tray")
            .with_icon(LaunchConfig::tray_icon(ICON))
            .build()
            .unwrap()
    };
    let tray_handler = move |ev, mut ctx: RendererContext| match ev {
        TrayEvent::Menu(MenuEvent { id }) if id == "3" => {
            ctx.launch_window(
                WindowConfig::new(AppComponent::new(App { radio_station })).with_size(500., 450.),
            );
        }
        TrayEvent::Menu(MenuEvent { id }) if id == "4" => {
            radio_station
                .write_channel(DataChannel::SlidersUpdate)
                .sliders
                .push(SliderData {
                    name: "New slider".to_string(),
                    volume: 50,
                    set_volume_action: VolumeAction::Print,
                });
        }
        TrayEvent::Menu(MenuEvent { id }) if id == "5" => {
            ctx.exit();
        }
        _ => {}
    };

    launch(
        LaunchConfig::new()
            .with_future(move |_| async move {
                let (state_tx, mut state_rx) = futures_channel::mpsc::unbounded::<ChannelSend>();
                let (serial_out_tx, mut serial_out_rx) =
                    futures_channel::mpsc::unbounded::<CommandsOut>();
                let serial_port: Arc<Mutex<Option<Box<dyn SerialPort>>>> =
                    Arc::new(Mutex::new(None));

                radio_station
                    .write_channel(DataChannel::NoUpdate)
                    .serial_out_tx = Some(serial_out_tx.clone());

                let serial_port_clone = serial_port.clone();
                let state_tx_clone = state_tx.clone();
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

                                    serial_port_clone.lock().unwrap().replace(port);

                                    state_tx_clone
                                        .unbounded_send(ChannelSend::DeviceInfoUpdate(Some(
                                            DeviceInfo {
                                                usb_info: port_info.1,
                                            },
                                        )))
                                        .expect("Failed to send device info");

                                    state_tx_clone
                                        .unbounded_send(ChannelSend::SlidersInfoUpdate(vec![
                                            SliderData {
                                                name: "Channel 1".to_string(),
                                                volume: 0,
                                                set_volume_action: VolumeAction::Print,
                                            },
                                            SliderData {
                                                name: "Channel 2".to_string(),
                                                volume: 0,
                                                set_volume_action: VolumeAction::Print,
                                            },
                                        ]))
                                        .expect("Failed to send sliders info");

                                    break;
                                }
                                Err(e) => {
                                    println!("Device not found! {}", e);
                                }
                            };
                            Timer::after(Duration::from_secs(1)).await;
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
                                    if let Some(port) =
                                        &mut serial_port_clone_2.lock().unwrap().as_mut()
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
                let state_tx_clone2 = state_tx.clone();
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
                                        let _ = state_tx_clone2.unbounded_send(ChannelSend::DeviceInfoUpdate(None));
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
                                                                i + 1,
                                                                SliderData {
                                                                    name: slider
                                                                        .name
                                                                        .clone(),
                                                                    volume: 0,
                                                                    set_volume_action: VolumeAction::Print,
                                                                },
                                                            ),
                                                        );
                                                    }
                                                }
                                                CommandsIn::SendVolume(volume_info) => {
                                                    println!("Received volume info: {:?}", volume_info);
                                                    let _ = state_tx.unbounded_send(
                                                        ChannelSend::SliderVolumeUpdate(
                                                            volume_info.channel.into(),
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

                while let Some(channel_data) = state_rx.next().await {
                    match channel_data {
                        ChannelSend::SlidersInfoUpdate(sliders) => {
                            radio_station
                                .write_channel(DataChannel::SlidersUpdate)
                                .sliders = sliders;
                        }
                        ChannelSend::SliderInfoUpdate(index, slider_data) => {
                            radio_station
                                .write_channel(DataChannel::SlidersUpdate)
                                .sliders[index] = slider_data;
                        }
                        ChannelSend::DeviceInfoUpdate(device_info) => {
                            // let router_context = RouterContext::get();
                            // if router_context.current::<Route>() == Route::Loading
                            //     && device_info.is_some()
                            // {
                            //     router_context.replace(Route::Main);
                            // } else if router_context.current::<Route>() != Route::Loading
                            //     && device_info.is_none()
                            // {
                            //     router_context.replace(Route::Loading);
                            // }

                            radio_station
                                .write_channel(DataChannel::DeviceInfo)
                                .device_info = device_info;
                        }
                        ChannelSend::SliderVolumeUpdate(channel, volume) => {
                            radio_station
                                .write_channel(DataChannel::SlidersUpdate)
                                .sliders[channel - 1]
                                .volume = volume;
                            run_action(&radio_station.read().sliders[channel - 1]);
                        },
                    }
                }
            })
            .with_tray(tray_icon, tray_handler)
            .with_window(WindowConfig::new(AppComponent::new(App { radio_station }))),
    )
}

#[allow(dead_code)]
pub struct DeviceInfo {
    pub usb_info: UsbPortInfo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderData {
    pub name: String,
    pub volume: u8,
    pub set_volume_action: VolumeAction,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VolumeAction {
    Print,
}

#[derive(Default)]
struct Data {
    pub device_info: Option<DeviceInfo>,
    pub sliders: Vec<SliderData>,
    pub serial_out_tx: Option<UnboundedSender<CommandsOut>>,
}

#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
pub enum DataChannel {
    SlidersUpdate,
    DeviceInfo,
    NoUpdate,
}

impl RadioChannel<Data> for DataChannel {}

pub enum ChannelSend {
    DeviceInfoUpdate(Option<DeviceInfo>),
    SliderVolumeUpdate(usize, u8),
    SlidersInfoUpdate(Vec<SliderData>),
    SliderInfoUpdate(usize, SliderData),
}
