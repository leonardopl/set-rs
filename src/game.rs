#[cfg(not(feature = "web"))]
use std::time::Instant;
#[cfg(feature = "web")]
use web_time::Instant;

use rand::seq::SliceRandom;
use ratatui::layout::Rect;

const FEEDBACK_TICKS: u8 = 90; // 1.5s at 60Hz
const AUTO_SELECT_TICKS: u8 = 180; // ~3s at 60Hz

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonAction {
    NewGame,
    Quit,
    Hint,
    AutoSelect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Circle,
    Square,
    Triangle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    Solid,
    Empty,
    Striped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Number {
    One,
    Two,
    Three,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetResult {
    Valid,
    Invalid,
}

macro_rules! impl_index {
    ($t:ty, $a:ident, $b:ident, $c:ident) => {
        impl $t {
            pub fn from_index(i: u8) -> Self {
                match i {
                    0 => Self::$a,
                    1 => Self::$b,
                    _ => Self::$c,
                }
            }

            pub fn as_index(self) -> u8 {
                match self {
                    Self::$a => 0,
                    Self::$b => 1,
                    Self::$c => 2,
                }
            }
        }
    };
}

impl_index!(Color, Red, Green, Blue);
impl_index!(Shape, Circle, Square, Triangle);
impl_index!(Fill, Solid, Empty, Striped);
impl_index!(Number, One, Two, Three);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card {
    pub color: Color,
    pub shape: Shape,
    pub fill: Fill,
    pub number: Number,
}

impl Card {
    pub fn new(color: Color, shape: Shape, fill: Fill, number: Number) -> Self {
        Self { color, shape, fill, number }
    }

}

pub fn is_valid_set(a: &Card, b: &Card, c: &Card) -> bool {
    let valid_attr = |x: u8, y: u8, z: u8| -> bool {
        (x == y && y == z) || (x != y && y != z && x != z)
    };

    valid_attr(a.color.as_index(), b.color.as_index(), c.color.as_index())
        && valid_attr(a.shape.as_index(), b.shape.as_index(), c.shape.as_index())
        && valid_attr(a.fill.as_index(), b.fill.as_index(), c.fill.as_index())
        && valid_attr(a.number.as_index(), b.number.as_index(), c.number.as_index())
}

pub const MIN_CARD_WIDTH: u16 = 10;
pub const MAX_CARD_WIDTH: u16 = 16;
pub const MIN_CARD_HEIGHT: u16 = 11;
pub const MAX_CARD_HEIGHT: u16 = 12;
pub const NUM_ROWS: usize = 3;

/// Computes the number of columns for a board with the given card count.
pub fn desired_cols(board_len: usize) -> usize {
    board_len.div_ceil(NUM_ROWS).max(1)
}

/// Returns the grid dimensions and card count that fit on one page at minimum card sizes.
pub fn cards_per_page(width: u16, height: u16) -> (usize, usize, usize) {
    let cols = (width / MIN_CARD_WIDTH).max(1) as usize;
    let rows = ((height / MIN_CARD_HEIGHT) as usize).clamp(1, NUM_ROWS);
    (cols, rows, cols * rows)
}

/// Returns the total number of pages needed to display all cards.
pub fn total_pages(board_len: usize, width: u16, height: u16) -> usize {
    let (_, _, per_page) = cards_per_page(width, height);
    if per_page == 0 { 1 } else { board_len.div_ceil(per_page).max(1) }
}

pub fn find_set_in(board: &[Card]) -> Option<(usize, usize, usize)> {
    let len = board.len();
    for i in 0..len {
        for j in (i + 1)..len {
            for k in (j + 1)..len {
                if is_valid_set(&board[i], &board[j], &board[k]) {
                    return Some((i, j, k));
                }
            }
        }
    }
    None
}

fn generate_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(81);
    for c in 0..3u8 {
        for s in 0..3u8 {
            for f in 0..3u8 {
                for n in 0..3u8 {
                    deck.push(Card::new(
                        Color::from_index(c),
                        Shape::from_index(s),
                        Fill::from_index(f),
                        Number::from_index(n),
                    ));
                }
            }
        }
    }
    deck
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputMethod {
    #[default]
    Keyboard,
    Mouse,
}

pub struct Game {
    pub board: Vec<Card>,
    pub deck: Vec<Card>,
    pub focus: usize,
    pub hover: Option<usize>,
    pub selected: Vec<usize>,
    pub score: usize,
    pub last_result: Option<SetResult>,
    pub feedback_ticks_remaining: u8,
    pub last_checked: Vec<usize>,
    pub show_focus: bool,
    pub last_input: InputMethod,
    pub card_areas: Vec<Rect>,
    pub button_areas: Vec<(ButtonAction, Rect)>,
    pub hint: Vec<usize>,
    pub auto_select_ticks: u8,
    pub turn_start: Instant,
    pub term_cols: u16,
    pub term_rows: u16,
    pub scroll_page: usize,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        let mut deck = generate_deck();
        deck.shuffle(&mut rand::rng());

        let mut board: Vec<Card> = deck.split_off(deck.len() - 12);

        // Auto-deal extra cards if no valid SET exists on the initial board
        while !deck.is_empty() && find_set_in(&board).is_none() {
            for _ in 0..3 {
                if let Some(card) = deck.pop() {
                    board.push(card);
                }
            }
        }

        Self {
            board,
            deck,
            focus: 0,
            hover: None,
            selected: Vec::new(),
            score: 0,
            last_result: None,
            feedback_ticks_remaining: 0,
            last_checked: Vec::new(),
            show_focus: false,
            last_input: InputMethod::default(),
            card_areas: Vec::new(),
            button_areas: Vec::new(),
            hint: Vec::new(),
            auto_select_ticks: 0,
            turn_start: Instant::now(),
            term_cols: 0,
            term_rows: 0,
            scroll_page: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.auto_select_ticks > 0 {
            self.auto_select_ticks -= 1;
            if self.auto_select_ticks == 0 && self.selected.len() == 3 {
                self.check_selection();
            }
        }
        if self.feedback_ticks_remaining > 0 {
            self.feedback_ticks_remaining -= 1;
            if self.feedback_ticks_remaining == 0 {
                self.last_result = None;
                self.last_checked.clear();
            }
        }
    }

    pub fn move_focus(&mut self, dx: i32, dy: i32) {
        self.last_input = InputMethod::Keyboard;
        self.show_focus = true;

        let (page_cols, _, _) = cards_per_page(self.term_cols, self.term_rows);
        if page_cols == 0 {
            return;
        }


        let logical_cols = desired_cols(self.board.len());
        let total_rows = self.board.len().div_ceil(logical_cols) as i32;
        let current_col = (self.focus % logical_cols) as i32;
        let current_row = (self.focus / logical_cols) as i32;

        let new_col = (current_col + dx).rem_euclid(logical_cols as i32);
        let new_row = (current_row + dy).rem_euclid(total_rows);

        let new_focus = (new_row as usize) * logical_cols + (new_col as usize);
        if new_focus < self.board.len() {
            self.focus = new_focus;
            self.ensure_focus_visible();
        }
    }

    fn ensure_focus_visible(&mut self) {
        let (_, _, per_page) = cards_per_page(self.term_cols, self.term_rows);
        if per_page == 0 {
            return;
        }
        let focus_page = self.focus / per_page;
        self.scroll_page = focus_page;
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_page > 0 {
            self.scroll_page -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        let max_page = total_pages(self.board.len(), self.term_cols, self.term_rows).saturating_sub(1);
        if self.scroll_page < max_page {
            self.scroll_page += 1;
        }
    }

    pub fn update_hover(&mut self, x: u16, y: u16) {
        self.last_input = InputMethod::Mouse;
        self.hover = self.card_areas.iter().position(|rect| {
            x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
        });
    }

    pub fn toggle_selection(&mut self) {
        if self.auto_select_ticks > 0 {
            return;
        }
        self.hint.clear();
        let idx = self.active_card_index();
        if let Some(idx) = idx {
            if idx >= self.board.len() {
                return;
            }
            if let Some(pos) = self.selected.iter().position(|&i| i == idx) {
                self.selected.remove(pos);
            } else if self.selected.len() < 3 {
                self.selected.push(idx);
                if self.selected.len() == 3 {
                    self.check_selection();
                }
            }
        }
    }

    fn check_selection(&mut self) {
        let indices = self.selected.clone();
        let a = &self.board[indices[0]];
        let b = &self.board[indices[1]];
        let c = &self.board[indices[2]];

        if is_valid_set(a, b, c) {
            self.score += 1;
            self.turn_start = Instant::now();
            self.last_result = Some(SetResult::Valid);
            self.feedback_ticks_remaining = FEEDBACK_TICKS;
            self.last_checked.clear();
            self.selected.clear();
            match self.last_input {
                InputMethod::Mouse => {
                    self.show_focus = false;
                    self.hover = None;
                }
                InputMethod::Keyboard => {}
            }
            self.remove_and_deal(indices[0], indices[1], indices[2]);
            self.auto_deal();
        } else {
            self.last_result = Some(SetResult::Invalid);
            self.feedback_ticks_remaining = FEEDBACK_TICKS;
            self.last_checked = indices;
            self.selected.clear();
        }
    }

    fn remove_and_deal(&mut self, i0: usize, i1: usize, i2: usize) {
        let mut indices = [i0, i1, i2];
        indices.sort_unstable_by(|a, b| b.cmp(a));

        for &idx in &indices {
            self.board.remove(idx);
        }

        while self.board.len() < 12 {
            if let Some(card) = self.deck.pop() {
                self.board.push(card);
            } else {
                break;
            }
        }

        self.scroll_page = 0;

        if !self.board.is_empty() && self.focus >= self.board.len() {
            self.focus = self.board.len() - 1;
        }
    }

    pub fn deck_remaining(&self) -> usize {
        self.deck.len()
    }

    pub fn find_set(&self) -> Option<(usize, usize, usize)> {
        find_set_in(&self.board)
    }

    pub fn board_has_set(&self) -> bool {
        self.find_set().is_some()
    }

    pub fn show_hint(&mut self) {
        if let Some((i, j, k)) = self.find_set() {
            let set_cards = [i, j, k];
            if !self.hint.iter().all(|h| set_cards.contains(h)) {
                self.hint.clear();
            }
            if self.hint.len() < 3 {
                for &idx in &set_cards {
                    if !self.hint.contains(&idx) {
                        self.hint.push(idx);
                        break;
                    }
                }
            }
        }
    }

    pub fn auto_select(&mut self) {
        if self.auto_select_ticks > 0 {
            return;
        }
        self.hint.clear();
        if let Some((i, j, k)) = self.find_set() {
            self.selected = vec![i, j, k];
            self.auto_select_ticks = AUTO_SELECT_TICKS;
        }
    }

    pub fn is_game_over(&self) -> bool {
        if !self.deck.is_empty() {
            return false;
        }
        !self.board_has_set()
    }

    fn auto_deal(&mut self) {
        while !self.board_has_set() && !self.deck.is_empty() {
            for _ in 0..3 {
                if let Some(card) = self.deck.pop() {
                    self.board.push(card);
                }
            }
        }
    }

    pub fn active_card_index(&self) -> Option<usize> {
        match self.last_input {
            InputMethod::Keyboard if self.show_focus => Some(self.focus),
            InputMethod::Mouse => self.hover,
            _ => None,
        }
    }

    pub fn is_active(&self, idx: usize) -> bool {
        self.active_card_index() == Some(idx)
    }

    pub fn is_selected(&self, idx: usize) -> bool {
        self.selected.contains(&idx)
    }

    pub fn set_card_areas(&mut self, areas: Vec<Rect>) {
        self.card_areas = areas;
    }

    pub fn set_button_areas(&mut self, areas: Vec<(ButtonAction, Rect)>) {
        self.button_areas = areas;
    }

    pub fn button_at(&self, x: u16, y: u16) -> Option<ButtonAction> {
        self.button_areas.iter().find_map(|(action, rect)| {
            if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height {
                Some(*action)
            } else {
                None
            }
        })
    }
}
