use crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent,
    MouseEventKind,
};

use crate::app::App;
use crate::game::ButtonAction;

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
            KeyCode::Char('h') => app.game.show_hint(),
            KeyCode::Char('f') => app.game.auto_select(),
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
            let (x, y) = (mouse_event.column, mouse_event.row);
            if let Some(action) = app.game.button_at(x, y) {
                match action {
                    ButtonAction::Quit => app.quit(),
                    ButtonAction::Hint => app.game.show_hint(),
                    ButtonAction::AutoSelect => app.game.auto_select(),
                }
            } else {
                app.game.update_hover(x, y);
                app.game.toggle_selection();
            }
        }
        _ => {}
    }
    Ok(())
}
