use freya::prelude::*;
use freya_radio::hooks::use_radio;

#[derive(PartialEq)]
pub struct Main {}
impl Render for Main {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio::<crate::app::Data, crate::app::DataChannel>(
            crate::app::DataChannel::DeviceUpdate,
        );
        let device = radio.read().device.clone().unwrap();
        let handle = device.open();

        rect()
            .expanded()
            .center()
            .child(label().text("Device connected..."))
            .child(label().text(format!("Device handle present: {:?}", device)))
            .maybe_child(match handle {
                Ok(_) => Some(label().text(format!("Device opened successfully! {:?}", device))),
                Err(e) => Some(label().text(format!("Failed to open device: {}", e))),
            })
        // .child(Button::new().on_press(|| {}))
    }
}
