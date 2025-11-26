use std::time::Duration;

use async_std::task::sleep;
use freya::prelude::*;
use freya_radio::prelude::*;
use freya_router::prelude::*;

use crate::{
    pages::{Loading, Main},
    utils::find_device,
};

#[derive(Default)]
#[allow(dead_code)]
enum ConnectionState {
    #[default]
    Searching,
    Connected,
}

#[derive(Default)]
#[allow(dead_code)]
struct Data {
    pub conncection_state: ConnectionState,
    pub port: Option<Box<dyn serialport::SerialPort>>,
}

#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
#[allow(dead_code)]
pub enum DataChannel {
    ConncectionStateUpdate,
    PortUpdate,
    NoUpdate,
}

impl RadioChannel<Data> for DataChannel {}

const TARGET_VID: u16 = 0x303A;
const TARGET_PID: u16 = 0x8145;
#[allow(non_snake_case)]
pub fn App() -> impl IntoElement {
    use_init_radio_station::<Data, DataChannel>(Data::default);

    let mut radio = use_radio::<Data, DataChannel>(DataChannel::PortUpdate);
    use_hook(|| {
        spawn(async move {
            loop {
                match find_device(TARGET_VID, TARGET_PID) {
                    Some(port_name) => {
                        let port = match serialport::new(&port_name, 115200)
                            .timeout(Duration::from_secs(5))
                            .open()
                        {
                            Ok(port) => port,
                            Err(e) => {
                                println!("Error opening port: {}", e);
                                break;
                            }
                        };

                        radio.write().port = Some(port);

                        RouterContext::get().replace(Route::Main);

                        break;
                    }
                    None => {
                        println!("Device not found");
                    }
                }
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
