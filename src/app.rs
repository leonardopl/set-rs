use std::io::stdout;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};

use crate::{
    event::{Event, EventHandler},
    game::Game,
    input,
};

pub struct App {
    pub running: bool,
    pub game: Game,
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            game: Game {},
        }
    }

    pub fn run(mut self) -> color_eyre::Result<()> {
        let mut terminal = ratatui::init();
        crossterm::execute!(stdout(), EnableMouseCapture)?;

        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = crossterm::execute!(stdout(), DisableMouseCapture);
            ratatui::restore();
            hook(info);
        }));

        let events = EventHandler::new();

        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events(&events)?;
        }

        let _ = std::panic::take_hook();
        let _ = crossterm::execute!(stdout(), DisableMouseCapture);
        ratatui::restore();
        Ok(())
    }

    pub fn handle_events(&mut self, events: &EventHandler) -> color_eyre::Result<()> {
        match events.next()? {
            Event::Tick => self.game.tick(),
            Event::Crossterm(event) => input::handle_crossterm_event(self, event)?,
        }
        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
