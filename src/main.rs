use freya::prelude::*;

mod app;
mod components;
mod pages;
mod utils;

use crate::app::App;

fn main() {
    launch(LaunchConfig::new().with_window(
        WindowConfig::new(App).with_size(1200.0, 800.0), // .with_title("Audiomixer"),
    ))
}
