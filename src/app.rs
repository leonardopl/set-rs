use crate::game::Game;
use crate::input;
use crate::ui::render_app;

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

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn new_game(&mut self) {
        self.game = Game::new();
    }
}

#[cfg(feature = "terminal")]
impl App {
    pub fn run(mut self) -> color_eyre::Result<()> {
        use std::cell::RefCell;
        use std::io::stdout;
        use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
        use ratatui::layout::Rect;
        use crate::event::EventHandler;
        use crate::game::ButtonAction;

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
            let size = terminal.size()?;
            self.game.term_cols = size.width;
            self.game.term_rows = size.height;
            self.game.set_card_areas(card_areas.borrow().clone());
            self.game.set_button_areas(btn_areas.borrow().clone());
            self.handle_events(&events)?;
        }

        let _ = std::panic::take_hook();
        let _ = crossterm::execute!(stdout(), DisableMouseCapture);
        ratatui::restore();
        Ok(())
    }

    fn handle_events(&mut self, events: &crate::event::EventHandler) -> color_eyre::Result<()> {
        use crate::event::Event;
        match events.next()? {
            Event::Tick => self.game.tick(),
            Event::Crossterm(event) => input::handle_crossterm_event(self, event)?,
        }
        Ok(())
    }
}

#[cfg(feature = "web")]
impl App {
    pub fn run(self) {
        use std::cell::RefCell;
        use std::rc::Rc;
        use ratzilla::backend::webgl2::WebGl2Backend;
        use ratzilla::ratatui::Terminal;
        use ratzilla::WebRenderer;
        use web_sys::wasm_bindgen::prelude::*;
        use web_sys::wasm_bindgen::JsCast;

        let backend = WebGl2Backend::new().expect("Failed to create WebGl2Backend");
        let terminal = Terminal::new(backend).expect("Failed to create terminal");

        let app = Rc::new(RefCell::new(self));

        let app_key = Rc::clone(&app);
        terminal.on_key_event(move |key_event| {
            input::handle_web_key_event(&mut app_key.borrow_mut(), key_event);
        });

        let app_mouse = Rc::clone(&app);
        terminal.on_mouse_event(move |mouse_event| {
            input::handle_web_mouse_event(&mut app_mouse.borrow_mut(), mouse_event);
        });

        // Wheel event listener for vertical scroll
        let app_wheel = Rc::clone(&app);
        let wheel_closure = Closure::<dyn Fn(web_sys::WheelEvent)>::new(move |event: web_sys::WheelEvent| {
            event.prevent_default();
            input::handle_web_wheel_event(&mut app_wheel.borrow_mut(), &event);
        });
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector("canvas").ok().flatten())
            .map(|el| {
                el.add_event_listener_with_callback(
                    "wheel",
                    wheel_closure.as_ref().unchecked_ref(),
                )
                .ok()
            });
        wheel_closure.forget();

        // Touch event listeners for swipe-to-scroll
        let touch_tracker = Rc::new(RefCell::new(input::TouchTracker::new()));

        let tracker_start = Rc::clone(&touch_tracker);
        let touchstart_closure = Closure::<dyn Fn(web_sys::TouchEvent)>::new(move |event: web_sys::TouchEvent| {
            if event.touches().length() == 1 {
                if let Some(touch) = event.touches().get(0) {
                    tracker_start.borrow_mut().on_touch_start(touch.client_x() as f64, touch.client_y() as f64);
                }
            }
        });
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector("canvas").ok().flatten())
            .map(|el| {
                el.add_event_listener_with_callback(
                    "touchstart",
                    touchstart_closure.as_ref().unchecked_ref(),
                )
                .ok()
            });
        touchstart_closure.forget();

        let tracker_end = Rc::clone(&touch_tracker);
        let app_touch = Rc::clone(&app);
        let touchend_closure = Closure::<dyn Fn(web_sys::TouchEvent)>::new(move |event: web_sys::TouchEvent| {
            if let Some(touch) = event.changed_touches().get(0) {
                let direction = tracker_end.borrow_mut().on_touch_end(touch.client_x() as f64, touch.client_y() as f64);
                if let Some(dir) = direction {
                    let mut app = app_touch.borrow_mut();
                    match dir {
                        input::SwipeDirection::Up => app.game.scroll_down(),
                        input::SwipeDirection::Down => app.game.scroll_up(),
                    }
                }
            }
        });
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector("canvas").ok().flatten())
            .map(|el| {
                el.add_event_listener_with_callback(
                    "touchend",
                    touchend_closure.as_ref().unchecked_ref(),
                )
                .ok()
            });
        touchend_closure.forget();

        let app_draw = Rc::clone(&app);
        terminal.draw_web(move |frame| {
            let mut app = app_draw.borrow_mut();
            app.game.tick();

            let area = frame.area();
            let (cards, buttons) = render_app(&app, area, frame.buffer_mut());
            app.game.set_card_areas(cards);
            app.game.set_button_areas(buttons);
            app.game.term_cols = area.width;
            app.game.term_rows = area.height;
        });
    }
}
