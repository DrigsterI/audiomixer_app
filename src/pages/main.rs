use freya::prelude::*;
use freya_radio::hooks::use_radio;

use crate::{
    app::{Data, DataChannel},
    components::Slider,
};

#[derive(PartialEq)]
pub struct Main {}
impl Render for Main {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio::<Data, DataChannel>(DataChannel::DeviceUpdate);
        let device_info = use_reactive(
            &radio
                .read()
                .device_info
                .clone(),
        );
        let tx_option = radio
            .read()
            .broadcast_tx
            .clone();

        rect()
            .width(Size::percent(100.0))
            .height(Size::percent(100.0))
            .children([
                rect()
                    .width(Size::Fill)
                    .height(Size::px(60.0))
                    .background(Color::from_hex("#FFFFFF").unwrap())
                    .child(
                        rect()
                            .height(Size::Fill)
                            .main_align(Alignment::Center)
                            .children([
                                label()
                                    .font_size(16.0)
                                    .font_weight(FontWeight::BOLD)
                                    .text(match &*device_info.read() {
                                        Some(device_info) => device_info
                                            .product
                                            .clone()
                                            .unwrap_or("Unknown Device".to_string()),
                                        None => "No device connected".to_string(),
                                    })
                                    .into(),
                                label()
                                    .font_size(16.0)
                                    .font_weight(FontWeight::BOLD)
                                    .text(match &*device_info.read() {
                                        Some(device_info) => device_info
                                            .serial_number
                                            .clone()
                                            .unwrap_or("No serial".to_string()),
                                        None => "".to_string(),
                                    })
                                    .into(),
                            ]),
                    )
                    .into(),
                rect()
                    .width(Size::Fill)
                    .height(Size::Fill)
                    .content(Content::Flex)
                    .direction(Direction::Horizontal)
                    .children(if let Some(tx) = tx_option {
                        vec![
                            Slider::new()
                                .width(Size::flex(1.0))
                                .title("Master")
                                .on_change({
                                    let tx = tx.clone();
                                    move |value| on_changed(0, value, tx.clone())
                                })
                                .into(),
                            Slider::new()
                                .width(Size::flex(1.0))
                                .title("Game")
                                .on_change({
                                    let tx = tx.clone();
                                    move |value| on_changed(1, value, tx.clone())
                                })
                                .into(),
                            Slider::new()
                                .width(Size::flex(1.0))
                                .title("Chat")
                                .on_change({
                                    let tx = tx.clone();
                                    move |value| on_changed(2, value, tx.clone())
                                })
                                .into(),
                            Slider::new()
                                .width(Size::flex(1.0))
                                .title("Media")
                                .on_change({
                                    let tx = tx.clone();
                                    move |value| on_changed(3, value, tx.clone())
                                })
                                .into(),
                            Slider::new()
                                .width(Size::flex(1.0))
                                .title("Mic")
                                .on_change({
                                    let tx = tx.clone();
                                    move |value| on_changed(4, value, tx.clone())
                                })
                                .into(),
                        ]
                    } else {
                        vec![
                            rect()
                                .center()
                                .child(
                                    label()
                                        .text("Initializing...")
                                        .font_size(24.0)
                                )
                                .into()
                        ]
                    })
                    .into(),
            ])
    }
}

fn on_changed(channel: u8, value: f64, tx: async_broadcast::Sender<crate::utils::Commands>) {
    println!("Channel: {}, Value: {}", channel, value);
    let command = crate::utils::Commands::SetVolume(crate::utils::SetVolumeProps {
        channel,
        volume: value as u8,
    });
    let _ = tx.try_broadcast(command);
}
