use freya::prelude::*;

#[derive(PartialEq)]
pub struct Loading {}
impl Render for Loading {
    fn render(&self) -> impl IntoElement {
        rect()
            .expanded()
            .center()
            .child(label().text("Looking for a device..."))
    }
}
