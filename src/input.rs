#[cfg(feature = "terminal")]
use crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent,
    MouseEventKind,
};

use crate::app::App;
use crate::game::ButtonAction;

#[cfg(feature = "terminal")]
pub fn handle_crossterm_event(app: &mut App, event: CrosstermEvent) -> color_eyre::Result<()> {
    match event {
        CrosstermEvent::Key(key_event) => handle_key_events(app, key_event)?,
        CrosstermEvent::Mouse(mouse_event) => handle_mouse_events(app, mouse_event)?,
        _ => {}
    }
    Ok(())
}

#[cfg(feature = "terminal")]
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

#[cfg(feature = "terminal")]
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

#[cfg(feature = "web")]
pub fn handle_web_key_event(app: &mut App, key_event: ratzilla::event::KeyEvent) {
    use ratzilla::event::KeyCode;
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') if key_event.ctrl => app.quit(),
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

#[cfg(feature = "web")]
pub fn handle_web_mouse_event(app: &mut App, mouse_event: ratzilla::event::MouseEvent) {
    use ratzilla::event::MouseEventKind;

    // Convert pixel coordinates to cell coordinates
    let (x, y) = pixel_to_cell(mouse_event.x, mouse_event.y, app.game.term_cols, app.game.term_rows);

    match mouse_event.event {
        MouseEventKind::Moved => {
            app.game.update_hover(x, y);
        }
        MouseEventKind::Pressed => {
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
}

#[cfg(feature = "web")]
fn pixel_to_cell(pixel_x: u32, pixel_y: u32, term_cols: u16, term_rows: u16) -> (u16, u16) {
    if term_cols == 0 || term_rows == 0 {
        return (0, 0);
    }

    let rect = canvas_rect();
    let cell_w = rect.2 / term_cols as f64;
    let cell_h = rect.3 / term_rows as f64;

    let x = ((pixel_x as f64) - rect.0) / cell_w;
    let y = ((pixel_y as f64) - rect.1) / cell_h;
    (x.max(0.0) as u16, y.max(0.0) as u16)
}

#[cfg(feature = "web")]
fn canvas_rect() -> (f64, f64, f64, f64) {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.query_selector("canvas").ok().flatten())
        .map(|el| {
            let rect = el.get_bounding_client_rect();
            (rect.left(), rect.top(), rect.width(), rect.height())
        })
        .unwrap_or((0.0, 0.0, 0.0, 0.0))
}
