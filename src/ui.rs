use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color as RatColor, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};

use crate::app::App;
use crate::game::{ButtonAction, Card, Game, SetResult};

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::default()
            .style(Style::default().bg(RatColor::Black))
            .render(area, buf);

        let layout = Layout::horizontal([
            Constraint::Percentage(85),
            Constraint::Fill(1),
        ])
        .split(area);

        render_board(&self.game, layout[0], buf);
        render_info(&self.game, layout[1], buf);
    }
}

pub fn render_app(app: &App, area: Rect, buf: &mut Buffer) -> (Vec<Rect>, Vec<(ButtonAction, Rect)>) {
    // Fill the entire area with black background
    Block::default()
        .style(Style::default().bg(RatColor::Black))
        .render(area, buf);

    let layout = Layout::horizontal([
        Constraint::Percentage(85),
        Constraint::Fill(1),
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
        .style(Style::default().bg(RatColor::Black));

    let inner = block.inner(area);
    block.render(area, buf);

    let mut lines: Vec<Line> = Vec::new();
    let mut button_areas: Vec<(ButtonAction, Rect)> = Vec::new();

    lines.push(Line::from(format!("Score: {}", game.score)));
    lines.push(Line::from(format!("Deck:  {}", game.deck_remaining())));
    lines.push(Line::from(""));

    match game.last_result {
        Some(SetResult::Valid) => {
            lines.push(Line::from(Span::styled(
                "Valid SET!",
                Style::default().fg(RatColor::LightGreen),
            )));
        }
        Some(SetResult::Invalid) => {
            lines.push(Line::from(Span::styled(
                "Not a SET!",
                Style::default().fg(RatColor::LightRed),
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
            Style::default().fg(RatColor::Yellow),
        )));
        lines.push(Line::from(""));
    }

    if game.can_deal_extra() {
        lines.push(Line::from(Span::styled(
            "No SET on board!",
            Style::default().fg(RatColor::Yellow),
        )));
        lines.push(Line::from(""));
    }

    if game.can_deal_extra() {
        let btn_line = lines.len() as u16;
        lines.push(Line::from(Span::styled(
            "[ Deal Extra ]",
            Style::default().fg(RatColor::Yellow),
        )));
        let btn_y = inner.y + btn_line;
        if btn_y < inner.y + inner.height {
            button_areas.push((ButtonAction::DealExtra, Rect {
                x: inner.x,
                y: btn_y,
                width: inner.width,
                height: 1,
            }));
        }
        lines.push(Line::from(""));
    }

    if !game.is_game_over() {
        let hint_line = lines.len() as u16;
        lines.push(Line::from(Span::styled(
            "[ Hint ]",
            Style::default().fg(RatColor::Cyan),
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
            "[ Auto Select ]",
            Style::default().fg(RatColor::Magenta),
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
    lines.push(Line::from(Span::styled(
        "[ Quit ]",
        Style::default().fg(RatColor::Red),
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
    lines.push(Line::from("e: deal extra"));
    lines.push(Line::from("q/Esc: quit"));

    Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .render(inner, buf);

    button_areas
}

fn render_board(game: &Game, area: Rect, buf: &mut Buffer) -> Vec<Rect> {
    let block = Block::bordered()
        .title("Pattern Matching Game")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(RatColor::Black));

    let inner = block.inner(area);
    block.render(area, buf);

    let num_rows = game.board.len().div_ceil(4).max(1) as u32;
    let row_constraints: Vec<Constraint> = (0..num_rows)
        .map(|_| Constraint::Ratio(1, num_rows))
        .collect();
    let rows = Layout::vertical(row_constraints).split(inner);

    let mut card_areas = Vec::with_capacity(game.board.len());

    for (row_idx, row_area) in rows.iter().enumerate() {
        let cols = Layout::horizontal([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(*row_area);

        for (col_idx, col_area) in cols.iter().enumerate() {
            let card_idx = row_idx * 4 + col_idx;
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
                    is_hinted: game.hint == Some(card_idx),
                    feedback,
                };
                card_widget.render(*col_area, buf);
            }
        }
    }

    card_areas
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
        use crate::game::{Color, Fill, Number, Shape};

        let symbol = match (self.card.shape(), self.card.fill()) {
            (Shape::Circle, Fill::Solid) => "●",
            (Shape::Circle, Fill::Empty) => "○",
            (Shape::Circle, Fill::Striped) => "◉",
            (Shape::Square, Fill::Solid) => "◼",
            (Shape::Square, Fill::Empty) => "◻",
            (Shape::Square, Fill::Striped) => "▣",
            (Shape::Diamond, Fill::Solid) => "◆",
            (Shape::Diamond, Fill::Empty) => "◇",
            (Shape::Diamond, Fill::Striped) => "◈",
        };

        let count = match self.card.number() {
            Number::One => 1,
            Number::Two => 2,
            Number::Three => 3,
        };

        let symbols: String = std::iter::repeat_n(symbol, count).collect::<Vec<_>>().join(" ");

        let color = match self.card.color() {
            Color::Red => RatColor::LightRed,
            Color::Green => RatColor::LightGreen,
            Color::Blue => RatColor::LightBlue,
        };

        // Border priority: feedback > selected > hinted > active > default
        let (border_type, border_style) = if let Some(result) = self.feedback {
            let fc = match result {
                SetResult::Valid => RatColor::LightGreen,
                SetResult::Invalid => RatColor::LightRed,
            };
            (BorderType::Double, Style::default().fg(fc))
        } else if self.is_selected {
            (BorderType::Double, Style::default().fg(RatColor::Green))
        } else if self.is_hinted {
            (BorderType::Double, Style::default().fg(RatColor::Cyan))
        } else if self.is_active {
            (BorderType::Double, Style::default().fg(RatColor::Yellow))
        } else {
            (BorderType::Rounded, Style::default())
        };

        let bg_color = RatColor::Black;

        let block = Block::bordered()
            .border_type(border_type)
            .border_style(border_style)
            .style(Style::default().bg(bg_color));

        let inner = block.inner(area);
        block.render(area, buf);

        for y in inner.y..inner.y + inner.height {
            for x in inner.x..inner.x + inner.width {
                buf[(x, y)].set_bg(bg_color);
            }
        }

        let paragraph = Paragraph::new(symbols)
            .style(Style::default().fg(color).bg(bg_color))
            .alignment(Alignment::Center);

        let text_area = Rect {
            x: inner.x,
            y: inner.y + inner.height / 2,
            width: inner.width,
            height: 1,
        };

        paragraph.render(text_area, buf);
    }
}
