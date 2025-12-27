use freya::prelude::*;
use freya_radio::hooks::use_radio;
use futures_channel::mpsc::UnboundedSender;

use crate::{Data, DataChannel, components::Slider, utils::CommandsOut};

#[derive(PartialEq)]
pub struct Main {}
impl Render for Main {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio::<Data, DataChannel>(DataChannel::DeviceInfoUpdate);
        let device_info = use_reactive(
            &radio
                .read()
                .device_info
                .usb_info
                .clone(),
        );
        let slider_radio = use_radio::<Data, DataChannel>(DataChannel::SlidersUpdate);
        let sliders = use_reactive(
            &slider_radio
                .read()
                .sliders
                .clone(),
        );
        let tx_option = radio
            .read()
            .serial_out_tx
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
                    .padding(8.0)
                    .spacing(8.0)
                    .children(if let Some(tx) = tx_option {
                        let mut i = 0;
                        sliders
                            .read()
                            .iter()
                            .map(|slider| {
                                i += 1;
                                Slider::new()
                                    .width(Size::flex(1.0))
                                    .title(
                                        slider
                                            .name
                                            .clone(),
                                    )
                                    .value(
                                        slider
                                            .volume
                                            .into(),
                                    )
                                    .on_change({
                                        let tx = tx.clone();
                                        move |value| on_changed(i, value, tx.clone())
                                    })
                                    .into()
                            })
                            .collect()
                    } else {
                        vec![]
                    })
                    .into(),
            ])
    }
}

fn on_changed(channel: u8, value: f64, tx: UnboundedSender<CommandsOut>) {
    let command = CommandsOut::SetVolume(crate::utils::SetVolumeProps {
        channel,
        volume: value as u8,
    });
    let _ = tx.unbounded_send(command);
}
