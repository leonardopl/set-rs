use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use color_eyre::eyre::WrapErr;
use crossterm::event::{self, Event as CrosstermEvent};

const TICKS_PER_SECOND: f64 = 60.0;

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Crossterm(CrosstermEvent),
}

#[derive(Debug)]
pub struct EventHandler {
    receiver: mpsc::Receiver<Event>,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let event_thread = EventThread::new(sender);
        thread::spawn(move || event_thread.run().expect("Event thread failed"));
        Self { receiver }
    }

    pub fn next(&self) -> color_eyre::Result<Event> {
        Ok(self.receiver.recv()?)
    }
}

struct EventThread {
    sender: mpsc::Sender<Event>,
}

impl EventThread {
    fn new(sender: mpsc::Sender<Event>) -> Self {
        Self { sender }
    }

    fn run(self) -> color_eyre::Result<()> {
        let tick_interval = Duration::from_secs_f64(1.0 / TICKS_PER_SECOND);
        let mut last_tick = Instant::now();

        loop {
            let timeout = tick_interval.saturating_sub(last_tick.elapsed());

            if event::poll(timeout).wrap_err("Failed to poll for crossterm events")? {
                let event = event::read().wrap_err("Failed to read crossterm event")?;
                if !self.send(Event::Crossterm(event)) {
                    break;
                }
            }

            let elapsed = last_tick.elapsed();
            if elapsed >= tick_interval {
                if elapsed > tick_interval * 2 {
                    last_tick = Instant::now();
                } else {
                    last_tick += tick_interval;
                }
                if !self.send(Event::Tick) {
                    break;
                }
            }
        }
        Ok(())
    }

    fn send(&self, event: Event) -> bool {
        self.sender.send(event).is_ok()
    }
}
