use freya::components::Slider as FreyaSlider;
use freya::prelude::*;
use std::borrow::Cow;

#[derive(PartialEq)]
pub struct Slider {
    title: Cow<'static, str>,
    width: Size,

    value: f64,

    on_changed: Option<EventHandler<f64>>,
}

impl Slider {
    pub fn new() -> Self {
        Self {
            title: Cow::from("Slider"),
            width: Size::default(),
            on_changed: None,
            value: 50.0,
        }
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;
        self
    }

    pub fn title(mut self, title: impl Into<Cow<'static, str>>) -> Self {
        self.title = title.into();
        self
    }

    pub fn on_change(mut self, on_changed: impl FnMut(f64) + 'static) -> Self {
        self.on_changed = Some(EventHandler::new(on_changed));
        self
    }

    pub fn value(mut self, value: f64) -> Self {
        self.value = value.clamp(0.0, 100.0);
        self
    }
}

impl Render for Slider {
    fn render(&self) -> impl IntoElement {
        let mut value = use_reactive(&self.value);

        use_side_effect({
            let on_changed = self
                .on_changed
                .clone();
            move || {
                if let Some(on_changed) = &on_changed {
                    on_changed.call(*value.read());
                }
            }
        });

        rect()
            .height(Size::Fill)
            .width(
                self.width
                    .clone(),
            )
            .center()
            .background(Color::from_hex("#464646").unwrap())
            .corner_radius(16.0)
            .content(Content::Flex)
            .children([
                rect()
                    .center()
                    .width(Size::Fill)
                    .padding(8.0)
                    .child(
                        label()
                            .font_size(36.0)
                            .font_weight(FontWeight::BOLD)
                            .text(
                                self.title
                                    .clone(),
                            ),
                    )
                    .into(),
                rect()
                    .height(Size::flex(1.0))
                    .padding(48.0)
                    .child(
                        FreyaSlider::new(move |arg| {
                            let val = arg.round();
                            if val != *value.read() {
                                value.set(val);
                            }
                        })
                        .size(Size::flex(1.0))
                        .value(value())
                        .background(Color::from_hex("#666666").unwrap())
                        .thumb_background(Color::from_hex("#1E1E1E").unwrap())
                        .thumb_inner_background(Color::from_hex("#1E1E1E").unwrap())
                        .direction(Direction::Vertical),
                    )
                    .into(),
                rect()
                    .center()
                    .width(Size::Fill)
                    .padding(8.0)
                    .child(
                        label()
                            .font_size(36.0)
                            .font_weight(FontWeight::BOLD)
                            .text(format!("{}%", value())),
                    )
                    .into(),
            ])
    }
}
