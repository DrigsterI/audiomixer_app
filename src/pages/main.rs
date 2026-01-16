use freya::{prelude::*, radio::use_radio};

use crate::{
    DataChannel,
    components::Slider,
    utils::{CommandsOut, SetVolumeProps, run_action},
};

#[derive(PartialEq)]
pub struct Main {}
impl Component for Main {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(DataChannel::SlidersUpdate);
        let tx_option = radio.read().serial_out_tx.clone();

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
                                    .text("Device")
                                    .into(),
                                label()
                                    .font_size(16.0)
                                    .font_weight(FontWeight::BOLD)
                                    .text(" - Audiomixer")
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
                    .children(
                        radio
                            .read()
                            .sliders
                            .iter()
                            .enumerate()
                            .map(|(index, slider)| {
                                Slider::new()
                                    .title(slider.name.clone())
                                    .width(Size::flex(1.0))
                                    .value(slider.volume as f64)
                                    .on_change({
                                        let slider_clone = slider.clone();
                                        let tx = tx_option.clone();
                                        move |val: f64| {
                                            radio.write().sliders[index].volume = val as u8;
                                            tx.clone()
                                                .expect("Sender for commands is not set")
                                                .unbounded_send(CommandsOut::SetVolume(
                                                    SetVolumeProps {
                                                        channel: index as u8 + 1,
                                                        volume: val as u8,
                                                    },
                                                ))
                                                .expect("Failed to send to channel");
                                            run_action(&radio.read().sliders[index]);
                                        }
                                    })
                                    .into_element()
                            }),
                    )
                    .into(),
            ])
    }
}
