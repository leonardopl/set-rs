// Circles: ● ○ ◉
// Squares: ◼ ◻ ▣
// Diamonds: ◆ ◇ ◈

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};

use crate::app::App;
use crate::game::{Game, Card};

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal([
            Constraint::Percentage(85),
            Constraint::Fill(1),
        ])
        .split(area);

        render_board(&self.game, layout[0], buf);
        render_info(layout[1], buf);
    }
}

/// Render the app and return card areas for mouse hit testing
pub fn render_app(app: &App, area: Rect, buf: &mut Buffer) -> Vec<Rect> {
    let layout = Layout::horizontal([
        Constraint::Percentage(85),
        Constraint::Fill(1),
    ])
    .split(area);

    let card_areas = render_board(&app.game, layout[0], buf);
    render_info(layout[1], buf);
    card_areas
}

fn render_info(area: Rect, buf: &mut Buffer) {
    let block = Block::bordered()
        .title("Info")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::HeavyDoubleDashed);

    let text = "Press `Esc`, `Ctrl-C`, `q`.";

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(block)
        .centered();

    paragraph.render(area, buf);
}

fn render_board(game: &Game, area: Rect, buf: &mut Buffer) -> Vec<Rect> {
    let block = Block::bordered()
        .title("Pattern Matching Game")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    block.render(area, buf);

    let rows = Layout::vertical([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .split(inner);

    let mut card_areas = Vec::with_capacity(12);

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
                let card_widget = CardWidget {
                    card: &game.board[card_idx],
                    is_active: game.is_active(card_idx),
                    is_selected: game.is_selected(card_idx),
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
}

impl Widget for CardWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use crate::game::{Color, Fill, Number, Shape};
        use ratatui::style::{Color as RatColor, Style};

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

        let (border_type, border_style) = if self.is_selected {
            (BorderType::Double, Style::default().fg(RatColor::Green))
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

        // Fill inner area with background
        for y in inner.y..inner.y + inner.height {
            for x in inner.x..inner.x + inner.width {
                buf[(x, y)].set_bg(bg_color);
            }
        }

        let paragraph = Paragraph::new(symbols)
            .style(Style::default().fg(color).bg(bg_color))
            .alignment(Alignment::Center);

        // Center vertically by calculating offset
        let text_area = Rect {
            x: inner.x,
            y: inner.y + inner.height / 2,
            width: inner.width,
            height: 1,
        };

        paragraph.render(text_area, buf);
    }
}