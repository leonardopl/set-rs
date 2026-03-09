use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color as RatColor, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::App;
use crate::game::{ButtonAction, Card, Game, MAX_CARD_HEIGHT, MAX_CARD_WIDTH, MIN_CARD_HEIGHT, MIN_CARD_WIDTH, NUM_ROWS, SetResult, cards_per_page, desired_cols, total_pages};

pub fn render_app(app: &App, area: Rect, buf: &mut Buffer) -> (Vec<Rect>, Vec<(ButtonAction, Rect)>) {
    Block::default()
        .style(Style::default().bg(RatColor::Rgb(0, 0, 0)))
        .render(area, buf);

    if area.width >= 70 {
        // Wide mode: board + sidebar centered together
        let sidebar_width: u16 = 14;
        let board_cols = desired_cols(app.game.board.len()) as u16;
        let max_board_width = board_cols * MAX_CARD_WIDTH;

        let layout = Layout::horizontal([
            Constraint::Max(max_board_width),
            Constraint::Length(sidebar_width),
        ])
        .flex(Flex::Center)
        .split(area);

        let card_areas = render_board(&app.game, layout[0], buf);
        let sidebar = if let Some(first) = card_areas.first() {
            let dy = first.y.saturating_sub(layout[1].y);
            Rect {
                y: layout[1].y + dy,
                height: layout[1].height.saturating_sub(dy),
                ..layout[1]
            }
        } else {
            layout[1]
        };
        let button_areas = render_info(&app.game, sidebar, buf);
        (card_areas, button_areas)
    } else {
        // Narrow mode: board top, compact info bottom
        let layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(5),
        ])
        .split(area);

        let card_areas = render_board(&app.game, layout[0], buf);
        let button_areas = render_info_compact(&app.game, layout[1], buf);
        (card_areas, button_areas)
    }
}

fn button_defs(game: &Game) -> Vec<(ButtonAction, &'static str, RatColor, RatColor)> {
    let mut buttons = Vec::new();
    if !game.is_game_over() {
        buttons.push((ButtonAction::Hint, " Hint (h) ", RatColor::Rgb(0, 0, 0), RatColor::Rgb(0, 210, 211)));
        buttons.push((ButtonAction::AutoSelect, " Auto (f) ", RatColor::Rgb(0, 0, 0), RatColor::Rgb(186, 85, 211)));
    }
    buttons.push((ButtonAction::NewGame, " New (n) ", RatColor::Rgb(0, 0, 0), RatColor::Rgb(80, 200, 120)));
    if !cfg!(feature = "web") {
        buttons.push((ButtonAction::Quit, " Quit (q) ", RatColor::Rgb(255, 255, 255), RatColor::Rgb(192, 57, 43)));
    }
    buttons
}

fn push_result_line<'a>(game: &Game, lines: &mut Vec<Line<'a>>) {
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
        None if game.is_game_over() => {
            lines.push(Line::from(Span::styled(
                "Game Over!",
                Style::default().fg(RatColor::Rgb(241, 196, 15)),
            )));
        }
        None => {
            lines.push(Line::from(""));
        }
    }
}

fn render_info(game: &Game, area: Rect, buf: &mut Buffer) -> Vec<(ButtonAction, Rect)> {
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    let mut lines: Vec<Line> = Vec::new();
    let mut button_areas: Vec<(ButtonAction, Rect)> = Vec::new();

    lines.push(Line::from(format!("Score: {}", game.score)));
    lines.push(Line::from(format!("Deck:  {}", game.deck_remaining())));
    let elapsed = game.turn_start.elapsed().as_secs();
    lines.push(Line::from(format!("Time:  {}:{:02}", elapsed / 60, elapsed % 60)));
    lines.push(Line::from(""));

    push_result_line(game, &mut lines);
    lines.push(Line::from(""));

    for (action, label, fg, bg) in &button_defs(game) {
        let btn_line = lines.len() as u16;
        lines.push(Line::from(Span::styled(
            *label,
            Style::default().fg(*fg).bg(*bg),
        )));
        let btn_y = inner.y + btn_line;
        if btn_y < inner.y + inner.height {
            button_areas.push((*action, Rect {
                x: inner.x,
                y: btn_y,
                width: inner.width,
                height: 1,
            }));
        }
        lines.push(Line::from(""));
    }

    Paragraph::new(lines).render(inner, buf);

    button_areas
}

fn render_info_compact(game: &Game, area: Rect, buf: &mut Buffer) -> Vec<(ButtonAction, Rect)> {
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    let mut lines: Vec<Line> = Vec::new();
    let mut button_areas: Vec<(ButtonAction, Rect)> = Vec::new();

    let elapsed = game.turn_start.elapsed().as_secs();
    lines.push(Line::from(format!("Score: {}  Deck: {}  Time: {}:{:02}",
        game.score, game.deck_remaining(), elapsed / 60, elapsed % 60)));

    push_result_line(game, &mut lines);

    let button_line = lines.len() as u16;
    let button_y = inner.y + button_line;
    let buttons = button_defs(game);

    let mut spans: Vec<Span> = Vec::new();
    let mut btn_positions: Vec<(ButtonAction, u16, u16)> = Vec::new();
    let mut col = 0u16;
    for (i, (action, label, fg, bg)) in buttons.iter().enumerate() {
        if i > 0 {
            spans.push(Span::from("  "));
            col += 2;
        }
        let label_len = label.len() as u16;
        btn_positions.push((*action, col, label_len));
        spans.push(Span::styled(*label, Style::default().fg(*fg).bg(*bg)));
        col += label_len;
    }
    lines.push(Line::from(spans));

    if button_y < inner.y + inner.height {
        let n = btn_positions.len();
        for i in 0..n {
            let (action, start_col, width) = btn_positions[i];
            let center = start_col + width / 2;

            let left = if i == 0 {
                0
            } else {
                let prev_center = btn_positions[i - 1].1 + btn_positions[i - 1].2 / 2;
                (prev_center + center) / 2
            };

            let right = if i == n - 1 {
                inner.width
            } else {
                let next_center = btn_positions[i + 1].1 + btn_positions[i + 1].2 / 2;
                (center + next_center) / 2
            };

            button_areas.push((action, Rect {
                x: inner.x + left,
                y: button_y,
                width: right.saturating_sub(left),
                height: 1,
            }));
        }
    }

    Paragraph::new(lines).render(inner, buf);

    button_areas
}

fn render_board(game: &Game, area: Rect, buf: &mut Buffer) -> Vec<Rect> {
    let inner = area;
    if inner.width < MIN_CARD_WIDTH || inner.height < MIN_CARD_HEIGHT {
        return vec![Rect::default(); game.board.len()];
    }

    // Check whether the ideal 3-row grid fits at minimum card sizes
    let ideal_cols = desired_cols(game.board.len());
    let fits_width = ideal_cols as u16 * MIN_CARD_WIDTH <= inner.width;
    let fits_height = NUM_ROWS as u16 * MIN_CARD_HEIGHT <= inner.height;
    let all_fit = fits_width && fits_height;


    let (page_cols, page_rows, per_page, num_pages, scroll);
    if all_fit {
        // All cards fit in a single page
        page_cols = ideal_cols;
        page_rows = NUM_ROWS;
        per_page = page_cols * page_rows;
        num_pages = 1;
        scroll = 0;
    } else {
        // Paginate: pack as many cards as possible at minimum size
        let (pc, pr, pp) = cards_per_page(inner.width, inner.height);
        page_cols = pc;
        page_rows = pr;
        per_page = pp;
        num_pages = total_pages(game.board.len(), inner.width, inner.height);
        scroll = game.scroll_page.min(num_pages.saturating_sub(1));
    }

    let page_start = scroll * per_page;
    let page_end = (page_start + per_page).min(game.board.len());
    let cards_on_page = page_end - page_start;


    let rows_needed = cards_on_page.div_ceil(page_cols).min(page_rows) as u16;
    if rows_needed == 0 {
        return vec![Rect::default(); game.board.len()];
    }

    // Distribute cards evenly across rows (e.g. 10 cards / 3 rows → 4-3-3 not 4-4-2)
    let rn = rows_needed as usize;
    let mut row_counts: Vec<usize> = Vec::with_capacity(rn);
    {
        let base = cards_on_page / rn;
        let extra = cards_on_page % rn;
        for i in 0..rn {
            row_counts.push(base + if i < extra { 1 } else { 0 });
        }
    }

    // Reserve space for scroll indicators when visible
    let top_reserve: u16 = if num_pages > 1 && scroll > 0 { 1 } else { 0 };
    let bot_reserve: u16 = if num_pages > 1 { 1 } else { 0 };
    let card_area = Rect {
        y: inner.y + top_reserve,
        height: inner.height.saturating_sub(top_reserve + bot_reserve),
        ..inner
    };


    let max_row_count = *row_counts.iter().max().unwrap_or(&page_cols);
    let card_w = (card_area.width / max_row_count as u16).min(MAX_CARD_WIDTH).max(MIN_CARD_WIDTH);
    let card_h = (card_area.height / rows_needed).clamp(MIN_CARD_HEIGHT, MAX_CARD_HEIGHT);

    let row_constraints: Vec<Constraint> = (0..rows_needed)
        .map(|_| Constraint::Length(card_h))
        .collect();
    let rows = Layout::vertical(row_constraints)
        .flex(Flex::Center)
        .split(card_area);

    // Initialize with zero-size rects; populated below for visible cards
    let mut card_areas = vec![Rect::default(); game.board.len()];

    let mut card_offset = 0usize;
    for (row_idx, row_area) in rows.iter().enumerate() {
        let row_card_count = row_counts[row_idx];
        if row_card_count == 0 {
            break;
        }

        let col_constraints: Vec<Constraint> =
            (0..row_card_count).map(|_| Constraint::Length(card_w)).collect();
        let cols = Layout::horizontal(col_constraints)
            .flex(Flex::Center)
            .split(*row_area);

        for (col_idx, col_area) in cols.iter().enumerate() {
            let card_idx = page_start + card_offset + col_idx;
            if card_idx < game.board.len() {
                card_areas[card_idx] = *col_area;
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
        card_offset += row_card_count;
    }


    if num_pages > 1 {
        let arrow_fg = RatColor::Rgb(0, 210, 211);
        let arrow_bg = RatColor::Rgb(0, 50, 55);
        let page_fg = RatColor::Rgb(220, 220, 220);
        let page_bg = RatColor::Rgb(40, 40, 50);


        if scroll > 0 {
            let arrow_str = "▲ ▲ ▲";
            let arrow_len = arrow_str.chars().count() as u16;
            let start_x = inner.x + inner.width.saturating_sub(arrow_len) / 2;
            let up_y = inner.y + 1;
            for (i, ch) in arrow_str.chars().enumerate() {
                let x = start_x + i as u16;
                if x < inner.x + inner.width && up_y < inner.y + inner.height {
                    buf[(x, up_y)]
                        .set_char(ch)
                        .set_fg(arrow_fg)
                        .set_bg(arrow_bg);
                }
            }
        }

        // Bottom line: combined down arrow + page counter, or page counter alone
        let bottom_y = inner.y + inner.height - 1;
        if bottom_y > inner.y {
            let bottom_text = if scroll + 1 < num_pages {
                format!("▼  {} / {}  ▼", scroll + 1, num_pages)
            } else {
                format!(" {} / {} ", scroll + 1, num_pages)
            };
            let text_len = bottom_text.chars().count() as u16;
            let start_x = inner.x + inner.width.saturating_sub(text_len) / 2;
            for (i, ch) in bottom_text.chars().enumerate() {
                let x = start_x + i as u16;
                if x < inner.x + inner.width {
                    let (fg, bg) = if ch == '▼' {
                        (arrow_fg, arrow_bg)
                    } else {
                        (page_fg, page_bg)
                    };
                    buf[(x, bottom_y)]
                        .set_char(ch)
                        .set_fg(fg)
                        .set_bg(bg);
                }
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

// Packs two pixel rows into one terminal row using half-block characters
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
