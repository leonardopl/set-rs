use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color as RatColor, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::App;
use crate::game::{ButtonAction, Card, Game, SetResult};

pub fn render_app(app: &App, area: Rect, buf: &mut Buffer) -> (Vec<Rect>, Vec<(ButtonAction, Rect)>) {
    Block::default()
        .style(Style::default().bg(RatColor::Rgb(0, 0, 0)))
        .render(area, buf);

    let layout = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(25),
    ])
    .split(area);

    let card_areas = render_board(&app.game, layout[0], buf);
    let button_areas = render_info(&app.game, layout[1], buf);
    (card_areas, button_areas)
}

fn render_info(game: &Game, area: Rect, buf: &mut Buffer) -> Vec<(ButtonAction, Rect)> {
    let block = Block::bordered()
        .title("Info")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Plain)
        .padding(ratatui::widgets::Padding::uniform(1))
        .style(Style::default().bg(RatColor::Rgb(0, 0, 0)));

    let inner = block.inner(area);
    block.render(area, buf);

    let mut lines: Vec<Line> = Vec::new();
    let mut button_areas: Vec<(ButtonAction, Rect)> = Vec::new();

    lines.push(Line::from(format!("Score: {}", game.score)));
    lines.push(Line::from(format!("Deck:  {}", game.deck_remaining())));
    let elapsed = game.turn_start.elapsed().as_secs();
    lines.push(Line::from(format!("Time:  {}:{:02}", elapsed / 60, elapsed % 60)));
    lines.push(Line::from(""));

    match game.last_result {
        Some(SetResult::Valid) => {
            lines.push(Line::from(Span::styled(
                "Valid SET!",
                Style::default().fg(RatColor::Rgb(80, 200, 120)),
            )));
        }
        Some(SetResult::Invalid) => {
            lines.push(Line::from(Span::styled(
                "Not a SET!",
                Style::default().fg(RatColor::Rgb(255, 107, 107)),
            )));
        }
        None => {
            lines.push(Line::from(""));
        }
    }

    lines.push(Line::from(""));

    if game.is_game_over() {
        lines.push(Line::from(Span::styled(
            "Game Over!",
            Style::default().fg(RatColor::Rgb(241, 196, 15)),
        )));
        lines.push(Line::from(""));
    }

    if !game.is_game_over() {
        let hint_line = lines.len() as u16;
        lines.push(Line::from(Span::styled(
            "  Hint  ",
            Style::default().fg(RatColor::Rgb(0, 0, 0)).bg(RatColor::Rgb(0, 210, 211)),
        )));
        let hint_y = inner.y + hint_line;
        if hint_y < inner.y + inner.height {
            button_areas.push((ButtonAction::Hint, Rect {
                x: inner.x,
                y: hint_y,
                width: inner.width,
                height: 1,
            }));
        }
        lines.push(Line::from(""));

        let auto_line = lines.len() as u16;
        lines.push(Line::from(Span::styled(
            "  Auto Select  ",
            Style::default().fg(RatColor::Rgb(0, 0, 0)).bg(RatColor::Rgb(186, 85, 211)),
        )));
        let auto_y = inner.y + auto_line;
        if auto_y < inner.y + inner.height {
            button_areas.push((ButtonAction::AutoSelect, Rect {
                x: inner.x,
                y: auto_y,
                width: inner.width,
                height: 1,
            }));
        }
        lines.push(Line::from(""));
    }

    let quit_line = lines.len() as u16;
    let (quit_label, quit_fg, quit_bg) = if cfg!(feature = "web") {
        ("  New Game  ", RatColor::Rgb(0, 0, 0), RatColor::Rgb(80, 200, 120))
    } else {
        ("  Quit  ", RatColor::Rgb(255, 255, 255), RatColor::Rgb(192, 57, 43))
    };
    lines.push(Line::from(Span::styled(
        quit_label,
        Style::default().fg(quit_fg).bg(quit_bg),
    )));
    let quit_y = inner.y + quit_line;
    if quit_y < inner.y + inner.height {
        button_areas.push((ButtonAction::Quit, Rect {
            x: inner.x,
            y: quit_y,
            width: inner.width,
            height: 1,
        }));
    }

    lines.push(Line::from(""));
    lines.push(Line::from("Arrows/WASD: move"));
    lines.push(Line::from("Enter/Space: select"));
    lines.push(Line::from("h: hint"));
    lines.push(Line::from("f: auto select"));
    if cfg!(feature = "web") {
        lines.push(Line::from("q/Esc: new game"));
    } else {
        lines.push(Line::from("q/Esc: quit"));
    }

    Paragraph::new(lines).render(inner, buf);

    button_areas
}

fn render_board(game: &Game, area: Rect, buf: &mut Buffer) -> Vec<Rect> {
    let block = Block::bordered()
        .title("set-rs")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(RatColor::Rgb(0, 0, 0)));

    let inner = block.inner(area);
    block.render(area, buf);

    let num_cols = game.board.len().div_ceil(3).max(1);
    let num_rows = 3u32;
    let row_constraints: Vec<Constraint> = (0..num_rows)
        .map(|_| Constraint::Ratio(1, num_rows))
        .collect();
    let rows = Layout::vertical(row_constraints).split(inner);

    let mut card_areas = Vec::with_capacity(game.board.len());

    for (row_idx, row_area) in rows.iter().enumerate() {
        let col_constraints: Vec<Constraint> =
            (0..num_cols).map(|_| Constraint::Max(16)).collect();
        let cols = Layout::horizontal(col_constraints)
            .flex(Flex::Center)
            .split(*row_area);

        for (col_idx, col_area) in cols.iter().enumerate() {
            let card_idx = row_idx * num_cols + col_idx;
            card_areas.push(*col_area);
            if card_idx < game.board.len() {
                let feedback = if game.last_checked.contains(&card_idx) {
                    game.last_result
                } else {
                    None
                };
                let card_widget = CardWidget {
                    card: &game.board[card_idx],
                    is_active: game.is_active(card_idx),
                    is_selected: game.is_selected(card_idx),
                    is_hinted: game.hint.contains(&card_idx),
                    feedback,
                };
                card_widget.render(*col_area, buf);
            }
        }
    }

    card_areas
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Outside,
    Border,
    Interior,
}

fn oval_mask(width: usize, height: usize) -> Vec<Vec<CellKind>> {
    let mut mask = vec![vec![CellKind::Outside; width]; height];
    let hw = width as f64 / 2.0;
    let hh = height as f64 / 2.0;
    for (r, row) in mask.iter_mut().enumerate().take(height) {
        for (c, cell) in row.iter_mut().enumerate().take(width) {
            let nx = (c as f64 + 0.5 - hw) / hw;
            let ny = (r as f64 + 0.5 - hh) / hh;
            if nx * nx + ny * ny <= 1.0 {
                *cell = CellKind::Interior;
            }
        }
    }
    // Mark border cells: interior cells adjacent to outside or edge
    let mut border = vec![vec![false; width]; height];
    for r in 0..height {
        for c in 0..width {
            if mask[r][c] != CellKind::Interior {
                continue;
            }
            let at_edge = r == 0 || r == height - 1 || c == 0 || c == width - 1;
            let has_outside_neighbor = !at_edge && [
                mask[r - 1][c], mask[r + 1][c], mask[r][c - 1], mask[r][c + 1],
            ].contains(&CellKind::Outside);
            if at_edge || has_outside_neighbor {
                border[r][c] = true;
            }
        }
    }
    for r in 0..height {
        for c in 0..width {
            if border[r][c] {
                mask[r][c] = CellKind::Border;
            }
        }
    }
    mask
}

fn rect_mask(width: usize, height: usize) -> Vec<Vec<CellKind>> {
    let mut mask = vec![vec![CellKind::Interior; width]; height];
    for (r, row) in mask.iter_mut().enumerate().take(height) {
        for (c, cell) in row.iter_mut().enumerate().take(width) {
            if r == 0 || r == height - 1 || c == 0 || c == width - 1 {
                *cell = CellKind::Border;
            }
        }
    }
    mask
}

/// Pointing-up triangle.
fn triangle_mask(width: usize, height: usize) -> Vec<Vec<CellKind>> {
    let mut mask = vec![vec![CellKind::Outside; width]; height];
    let w = width as f64;
    let h = height as f64;
    for (r, row) in mask.iter_mut().enumerate().take(height) {
        let frac = (r as f64 + 1.0) / h;
        let half_span = (frac * w) / 2.0;
        let center = w / 2.0;
        let left = ((center - half_span).floor() as usize).max(0);
        let right = ((center + half_span).ceil() as usize).min(width);
        for cell in row.iter_mut().take(right).skip(left) {
            *cell = CellKind::Interior;
        }
    }
    let mut border = vec![vec![false; width]; height];
    for r in 0..height {
        for c in 0..width {
            if mask[r][c] != CellKind::Interior {
                continue;
            }
            let at_edge = r == 0 || r == height - 1 || c == 0 || c == width - 1;
            let has_outside_neighbor = !at_edge && [
                mask[r - 1][c], mask[r + 1][c], mask[r][c - 1], mask[r][c + 1],
            ].contains(&CellKind::Outside);
            if at_edge || has_outside_neighbor {
                border[r][c] = true;
            }
        }
    }
    for r in 0..height {
        for c in 0..width {
            if border[r][c] {
                mask[r][c] = CellKind::Border;
            }
        }
    }
    mask
}

/// Returns the pixel color for the given cell kind and fill, or None for background.
fn pixel_color(
    kind: CellKind,
    fill: crate::game::Fill,
    card_color: RatColor,
    dim_color: RatColor,
    px: u16,
    py: u16,
) -> Option<RatColor> {
    use crate::game::Fill;
    match kind {
        CellKind::Outside => None,
        CellKind::Border => Some(card_color),
        CellKind::Interior => match fill {
            Fill::Solid => Some(card_color),
            Fill::Striped => {
                let checker = ((px as u32) + (py as u32)).is_multiple_of(2);
                Some(if checker { card_color } else { dim_color })
            }
            Fill::Empty => None, // outline only
        },
    }
}

/// Renders two pixel rows into one terminal row via half-block characters.
#[allow(clippy::too_many_arguments)]
fn render_shape_row(
    top_row: &[CellKind],
    bot_row: Option<&[CellKind]>,
    fill: crate::game::Fill,
    card_color: RatColor,
    dim_color: RatColor,
    buf: &mut Buffer,
    term_x: u16,
    term_y: u16,
    pixel_y_top: u16,
    inner: Rect,
) {
    let bg = RatColor::Rgb(0, 0, 0);
    for (i, &top_kind) in top_row.iter().enumerate() {
        let px = term_x + i as u16;
        if px >= inner.x + inner.width {
            break;
        }
        let top_color = pixel_color(top_kind, fill, card_color, dim_color, px, pixel_y_top);
        let bot_color = bot_row.and_then(|br| {
            pixel_color(br[i], fill, card_color, dim_color, px, pixel_y_top + 1)
        });

        let cell = &mut buf[(px, term_y)];
        match (top_color, bot_color) {
            (Some(tc), Some(bc)) => {
                if tc == bc {
                    cell.set_char(' ').set_bg(tc);
                } else {
                    // Use ▀ with fg=top, bg=bottom
                    cell.set_char('▀').set_fg(tc).set_bg(bc);
                }
            }
            (Some(tc), None) => {
                cell.set_char('▀').set_fg(tc).set_bg(bg);
            }
            (None, Some(bc)) => {
                cell.set_char('▄').set_fg(bc).set_bg(bg);
            }
            (None, None) => {
                cell.set_char(' ').set_bg(bg);
            }
        }
    }
}

struct CardWidget<'a> {
    card: &'a Card,
    is_active: bool,
    is_selected: bool,
    is_hinted: bool,
    feedback: Option<SetResult>,
}

impl Widget for CardWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use crate::game::{Color, Number, Shape};

        let count = match self.card.number {
            Number::One => 1usize,
            Number::Two => 2,
            Number::Three => 3,
        };

        let card_color = match self.card.color {
            Color::Red => RatColor::Rgb(255, 107, 107),
            Color::Green => RatColor::Rgb(80, 200, 120),
            Color::Blue => RatColor::Rgb(100, 149, 237),
        };

        let dim_color = match self.card.color {
            Color::Red => RatColor::Rgb(80, 30, 30),
            Color::Green => RatColor::Rgb(20, 65, 35),
            Color::Blue => RatColor::Rgb(30, 40, 80),
        };

        let fill = self.card.fill;

        // Border priority: feedback > selected > hinted > active > default
        let (border_type, border_style) = if let Some(result) = self.feedback {
            let (fc, bc) = match result {
                SetResult::Valid => (RatColor::Rgb(80, 200, 120), RatColor::Rgb(20, 60, 35)),
                SetResult::Invalid => (RatColor::Rgb(255, 107, 107), RatColor::Rgb(80, 30, 30)),
            };
            (BorderType::Double, Style::default().fg(fc).bg(bc))
        } else if self.is_selected {
            (BorderType::Double, Style::default().fg(RatColor::Rgb(46, 204, 113)).bg(RatColor::Rgb(15, 60, 35)))
        } else if self.is_hinted {
            (BorderType::Double, Style::default().fg(RatColor::Rgb(0, 210, 211)).bg(RatColor::Rgb(0, 60, 60)))
        } else if self.is_active {
            (BorderType::Double, Style::default().fg(RatColor::Rgb(241, 196, 15)).bg(RatColor::Rgb(70, 55, 5)))
        } else {
            (BorderType::Rounded, Style::default())
        };

        let bg_color = RatColor::Rgb(0, 0, 0);

        let block = Block::bordered()
            .border_type(border_type)
            .border_style(border_style)
            .style(Style::default().bg(bg_color));

        let inner = block.inner(area);
        block.render(area, buf);

        for y in inner.y..inner.y + inner.height {
            for x in inner.x..inner.x + inner.width {
                buf[(x, y)].set_char(' ').set_bg(bg_color);
            }
        }

        let iw = inner.width as usize;
        let ih = inner.height as usize;
        if iw < 3 || ih < 1 {
            return;
        }

        let shape_w = iw.saturating_sub(2).max(3);

        let (shape_pixel_h, gap): (usize, usize) = {
            let needed_2row = count * 2 + count.saturating_sub(1);
            let needed_2row_nogap = count * 2;
            let needed_1row = count + count.saturating_sub(1);
            if needed_2row <= ih {
                (4, 1)
            } else if needed_2row_nogap <= ih {
                (4, 0)
            } else if needed_1row <= ih {
                (2, 1)
            } else {
                (2, 0) // best effort, even if it overflows
            }
        };

        let shape_term_h = shape_pixel_h / 2;
        let total_term_h = count * shape_term_h + count.saturating_sub(1) * gap;
        let start_y = inner.y + (ih as u16).saturating_sub(total_term_h as u16) / 2;
        let start_x = inner.x + (iw as u16).saturating_sub(shape_w as u16) / 2;

        let shape = self.card.shape;
        let mask = match shape {
            Shape::Circle => oval_mask(shape_w, shape_pixel_h),
            Shape::Square => rect_mask(shape_w, shape_pixel_h),
            Shape::Triangle => triangle_mask(shape_w, shape_pixel_h),
        };

        for shape_idx in 0..count {
            let sy = start_y + (shape_idx * (shape_term_h + gap)) as u16;

            for term_row in 0..shape_term_h {
                let ty = sy + term_row as u16;
                if ty >= inner.y + inner.height {
                    break;
                }
                let pixel_row_top = term_row * 2;
                let pixel_row_bot = pixel_row_top + 1;

                let top = &mask[pixel_row_top];
                let bot = if pixel_row_bot < shape_pixel_h {
                    Some(mask[pixel_row_bot].as_slice())
                } else {
                    None
                };

                render_shape_row(
                    top,
                    bot,
                    fill,
                    card_color,
                    dim_color,
                    buf,
                    start_x,
                    ty,
                    (shape_idx * shape_pixel_h + pixel_row_top) as u16,
                    inner,
                );
            }
        }
    }
}
