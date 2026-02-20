use crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent,
    MouseEventKind,
};

use crate::app::App;

pub fn handle_crossterm_event(app: &mut App, event: CrosstermEvent) -> color_eyre::Result<()> {
    match event {
        CrosstermEvent::Key(key_event) => handle_key_events(app, key_event)?,
        CrosstermEvent::Mouse(mouse_event) => handle_mouse_events(app, mouse_event)?,
        _ => {}
    }
    Ok(())
}

pub fn handle_key_events(app: &mut App, key_event: KeyEvent) -> color_eyre::Result<()> {
    if key_event.kind == KeyEventKind::Press {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => app.quit(),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => app.quit(),
            _ => {}
        }
    }
    Ok(())
}

pub fn handle_mouse_events(app: &mut App, mouse_event: MouseEvent) -> color_eyre::Result<()> {
    if matches!(mouse_event.kind, MouseEventKind::Up(_)) {
        app.quit();
    }
    Ok(())
}
