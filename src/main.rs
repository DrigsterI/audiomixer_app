use freya::prelude::*;

mod app;
mod pages;
mod utils;

use crate::app::App;

fn main() {
    launch(LaunchConfig::new().with_window(WindowConfig::new(App)))
}
