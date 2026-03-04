use ratatui::layout::Rect;

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputMethod {
    #[default]
    Keyboard,
    Mouse,
}

pub struct Game {
    pub board: Vec<Card>,
    pub focus: usize,
    pub hover: Option<usize>,
    pub selected: Vec<usize>,
    pub last_input: InputMethod,
    pub card_areas: Vec<Rect>,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Self {
        let board: Vec<Card> = (0..12)
            .map(|i| {
                let color = match i % 3 {
                    0 => Color::Red,
                    1 => Color::Green,
                    _ => Color::Blue,
                };
                let shape = match (i / 3) % 3 {
                    0 => Shape::Circle,
                    1 => Shape::Square,
                    _ => Shape::Diamond,
                };
                let fill = match i % 3 {
                    0 => Fill::Solid,
                    1 => Fill::Empty,
                    _ => Fill::Striped,
                };
                let number = match i % 3 {
                    0 => Number::One,
                    1 => Number::Two,
                    _ => Number::Three,
                };
                Card::new(color, shape, fill, number)
            })
            .collect();

        Self {
            board,
            focus: 0,
            hover: None,
            selected: Vec::new(),
            last_input: InputMethod::default(),
            card_areas: Vec::new(),
        }
    }

    pub fn tick(&self) {}

    pub fn move_focus(&mut self, dx: i32, dy: i32) {
        self.last_input = InputMethod::Keyboard;
        let cols = 4;
        let rows = 3;
        let current_col = (self.focus % cols) as i32;
        let current_row = (self.focus / cols) as i32;

        let new_col = (current_col + dx).rem_euclid(cols as i32) as usize;
        let new_row = (current_row + dy).rem_euclid(rows) as usize;

        self.focus = new_row * cols + new_col;
    }

    pub fn update_hover(&mut self, x: u16, y: u16) {
        self.last_input = InputMethod::Mouse;
        self.hover = self.card_areas.iter().position(|rect| {
            x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
        });
    }

    pub fn toggle_selection(&mut self) {
        let idx = self.active_card_index();
        if let Some(idx) = idx {
            if let Some(pos) = self.selected.iter().position(|&i| i == idx) {
                self.selected.remove(pos);
            } else {
                self.selected.push(idx);
            }
        }
    }

    /// Get the currently active card index based on last input method
    pub fn active_card_index(&self) -> Option<usize> {
        match self.last_input {
            InputMethod::Keyboard => Some(self.focus),
            InputMethod::Mouse => self.hover,
        }
    }

    /// Check if a card is the active one (for rendering)
    pub fn is_active(&self, idx: usize) -> bool {
        self.active_card_index() == Some(idx)
    }

    pub fn is_selected(&self, idx: usize) -> bool {
        self.selected.contains(&idx)
    }

    pub fn set_card_areas(&mut self, areas: Vec<Rect>) {
        self.card_areas = areas;
    }
}