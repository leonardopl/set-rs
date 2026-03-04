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
            KeyCode::Left | KeyCode::Char('a') => app.game.move_focus(-1, 0),
            KeyCode::Right | KeyCode::Char('d') => app.game.move_focus(1, 0),
            KeyCode::Up | KeyCode::Char('w') => app.game.move_focus(0, -1),
            KeyCode::Down | KeyCode::Char('s') => app.game.move_focus(0, 1),
            KeyCode::Enter | KeyCode::Char(' ') => app.game.toggle_selection(),
            _ => {}
        }
    }
    Ok(())
}

pub fn handle_mouse_events(app: &mut App, mouse_event: MouseEvent) -> color_eyre::Result<()> {
    match mouse_event.kind {
        MouseEventKind::Moved => {
            app.game.update_hover(mouse_event.column, mouse_event.row);
        }
        MouseEventKind::Down(_) => {
            app.game.update_hover(mouse_event.column, mouse_event.row);
            app.game.toggle_selection();
        }
        _ => {}
    }
    Ok(())
}
