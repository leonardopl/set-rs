use std::{cell::RefCell, io::stdout};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::layout::Rect;
use crate::game::ButtonAction;

use crate::{
    event::{Event, EventHandler},
    game::Game,
    input,
    ui::render_app,
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
            game: Game::new(),
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

        let card_areas: RefCell<Vec<Rect>> = RefCell::new(Vec::new());
        let btn_areas: RefCell<Vec<(ButtonAction, Rect)>> = RefCell::new(Vec::new());

        while self.running {
            terminal.draw(|frame| {
                let (cards, buttons) = render_app(&self, frame.area(), frame.buffer_mut());
                *card_areas.borrow_mut() = cards;
                *btn_areas.borrow_mut() = buttons;
            })?;
            self.game.set_card_areas(card_areas.borrow().clone());
            self.game.set_button_areas(btn_areas.borrow().clone());
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
