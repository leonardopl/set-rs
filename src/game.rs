use rand::seq::SliceRandom;
use ratatui::layout::Rect;

const FEEDBACK_TICKS: u8 = 90; // 1.5s at 60Hz
const AUTO_SELECT_TICKS: u8 = 180; // ~3s at 60Hz

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonAction {
    Quit,
    DealExtra,
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
    Diamond,
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
impl_index!(Shape, Circle, Square, Diamond);
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

    pub fn color(&self) -> Color { self.color }
    pub fn shape(&self) -> Shape { self.shape }
    pub fn fill(&self) -> Fill { self.fill }
    pub fn number(&self) -> Number { self.number }
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
    pub last_input: InputMethod,
    pub card_areas: Vec<Rect>,
    pub button_areas: Vec<(ButtonAction, Rect)>,
    pub hint: Option<usize>,
    pub auto_select_ticks: u8,
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

        let board: Vec<Card> = deck.split_off(deck.len() - 12);

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
            last_input: InputMethod::default(),
            card_areas: Vec::new(),
            button_areas: Vec::new(),
            hint: None,
            auto_select_ticks: 0,
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
        let cols = 4i32;
        let rows = self.board.len().div_ceil(4) as i32;
        let current_col = (self.focus % cols as usize) as i32;
        let current_row = (self.focus / cols as usize) as i32;

        let new_col = (current_col + dx).rem_euclid(cols) as usize;
        let new_row = (current_row + dy).rem_euclid(rows) as usize;

        let new_focus = new_row * cols as usize + new_col;
        if new_focus < self.board.len() {
            self.focus = new_focus;
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
        self.hint = None;
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
            self.last_result = Some(SetResult::Valid);
            self.feedback_ticks_remaining = FEEDBACK_TICKS;
            self.last_checked.clear();
            self.selected.clear();
            self.remove_and_deal(indices[0], indices[1], indices[2]);
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

        if !self.board.is_empty() && self.focus >= self.board.len() {
            self.focus = self.board.len() - 1;
        }
    }

    pub fn deck_remaining(&self) -> usize {
        self.deck.len()
    }

    pub fn find_set(&self) -> Option<(usize, usize, usize)> {
        let len = self.board.len();
        for i in 0..len {
            for j in (i + 1)..len {
                for k in (j + 1)..len {
                    if is_valid_set(&self.board[i], &self.board[j], &self.board[k]) {
                        return Some((i, j, k));
                    }
                }
            }
        }
        None
    }

    pub fn board_has_set(&self) -> bool {
        self.find_set().is_some()
    }

    pub fn show_hint(&mut self) {
        self.hint = None;
        if let Some((i, _, _)) = self.find_set() {
            self.hint = Some(i);
        }
    }

    pub fn auto_select(&mut self) {
        if self.auto_select_ticks > 0 {
            return;
        }
        self.hint = None;
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

    pub fn can_deal_extra(&self) -> bool {
        !self.board_has_set() && !self.deck.is_empty()
    }

    pub fn deal_extra(&mut self) {
        if !self.can_deal_extra() {
            return;
        }
        for _ in 0..3 {
            if let Some(card) = self.deck.pop() {
                self.board.push(card);
            } else {
                break;
            }
        }
    }

    pub fn active_card_index(&self) -> Option<usize> {
        match self.last_input {
            InputMethod::Keyboard => Some(self.focus),
            InputMethod::Mouse => self.hover,
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
