use crate::{
    Data, DataChannel,
    pages::{Loading, Main},
};

use freya::{
    prelude::*,
    radio::{RadioStation, use_share_radio},
    router::prelude::{Routable, RouterConfig, router},
};

pub struct App {
    pub radio_station: RadioStation<Data, DataChannel>,
}

impl Component for App {
    fn render(&self) -> impl IntoElement {
        use_share_radio(move || self.radio_station);

        router::<Route>(|| RouterConfig::default().with_initial_path(Route::Loading))
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
