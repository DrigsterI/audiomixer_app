use crate::{
    Data, DataChannel,
    pages::{Loading, Main},
    utils::CommandsOut,
};
use freya::prelude::*;
use freya_radio::prelude::*;
use freya_router::prelude::*;
use futures_channel::mpsc::UnboundedSender;

pub struct App {
    pub radio_station: RadioStation<Data, DataChannel>,
    pub serial_out_tx: UnboundedSender<CommandsOut>,
}

impl Render for App {
    fn render(&self) -> impl IntoElement {
        use_share_radio(move || self.radio_station);

        router::<Route>(|| RouterConfig::default().with_initial_path(Route::Main))
    }
}

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Loading,
    #[route("/main")]
    Main,
}
