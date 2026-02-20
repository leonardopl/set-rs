use crate::app::App;

pub mod app;
pub mod event;
pub mod game;
pub mod input;
pub mod ui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    App::new().run()
}
