use std::time::Duration;

use crate::{
    pages::{Loading, Main},
    utils::find_device,
};
use async_std::task::sleep;
use freya::prelude::*;
use freya_radio::prelude::*;
use freya_router::prelude::*;
use rusb::{Context, Device};

#[derive(Default)]
#[allow(dead_code)]
pub struct Data {
    pub device: Option<Device<Context>>,
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
    use_hook(|| {
        spawn(async move {
            loop {
                match find_device(TARGET_VID, TARGET_PID) {
                    Some(device) => {
                        println!("Device found!");
                        radio.write().device = Some(device);
                        RouterContext::get().replace(Route::Main);
                        break;
                    }
                    None => {
                        println!("Device not found!");
                    }
                };
                sleep(Duration::from_secs(2)).await;
            }
        });
    });

    router::<Route>(|| RouterConfig::default().with_initial_path(Route::Loading))
}

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Loading,
    #[route("/main")]
    Main,
}
