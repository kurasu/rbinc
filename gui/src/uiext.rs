use eframe::egui::{Button, Response, Sense, Ui, Widget, WidgetText};

pub trait UiExt {
    fn button_with_enable(&mut self, text: impl Into<WidgetText>, enabled: bool) -> Response;
}

impl UiExt for Ui {
    fn button_with_enable(&mut self, text: impl Into<WidgetText>, enabled: bool) -> Response {
        let mut button = Button::new(text);
        if !enabled {
            button = button.sense(Sense::empty())
        }
        button.ui(self)
    }
}
