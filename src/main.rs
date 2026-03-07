use crate::app::App;

pub mod app;
#[cfg(feature = "terminal")]
pub mod event;
pub mod game;
pub mod input;
pub mod ui;

#[cfg(feature = "terminal")]
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    App::new().run()
}

#[cfg(feature = "web")]
fn main() {
    App::new().run();
}
