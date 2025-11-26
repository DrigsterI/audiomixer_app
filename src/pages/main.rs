use freya::prelude::*;

#[derive(PartialEq)]
pub struct Main {}
impl Render for Main {
    fn render(&self) -> impl IntoElement {
        rect()
            .expanded()
            .center()
            .child(label().text("Device connected..."))
    }
}
