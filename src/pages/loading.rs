use freya::{prelude::*, radio::use_radio};
use freya_router::prelude::RouterContext;

use crate::{Data, DataChannel, app::Route};

#[derive(PartialEq)]
pub struct Loading {}
impl Component for Loading {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio::<Data, DataChannel>(DataChannel::DeviceInfo);

        use_side_effect(move || {
            if radio.read().device_info.is_none() {
                return;
            }
            RouterContext::get().replace(Route::Main);
        });

        rect()
            .expanded()
            .center()
            .child(label().text("Looking for a device..."))
    }
}
