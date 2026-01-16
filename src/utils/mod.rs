use crate::{SliderData, VolumeAction};

mod serial;
pub use serial::*;

pub fn run_action(slider_data: &SliderData) {
    match slider_data.set_volume_action {
        VolumeAction::Print => {
            println!(
                "Printing volume for {}: {}",
                slider_data.name, slider_data.volume
            );
        }
    }
}
