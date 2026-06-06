//! OpenCAN GUI Application

mod state;
mod views;
mod backend;
mod app;
mod helpers;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    iced::application("OpenCAN", state::App::update, state::App::view)
        .theme(state::App::theme)
        .subscription(state::App::subscription)
        .run_with(state::App::new)
}
