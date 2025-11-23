use iced::widget::{column, text};
use iced::{Element, Sandbox, Settings};

pub struct App;

#[derive(Debug, Clone, Copy)]
pub enum Message {}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        Self
    }

    fn title(&self) -> String {
        String::from("RustSpy")
    }

    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<'_, Message> {
        column![text("RustSpy - Placeholder UI")].into()
    }
}

pub fn run() -> iced::Result {
    App::run(Settings::default())
}
