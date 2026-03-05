# pattern-match-game

A terminal-based implementation of the [SET card game](https://en.wikipedia.org/wiki/Set_(card_game)) built with Rust and [ratatui](https://ratatui.rs).

Personal project built to learn Rust, focusing on TUI rendering, game logic, and event-driven architecture.

<!-- ![screenshot](assets/screenshot.png) -->

## Features

- Full 81-card SET deck (4 attributes: color, shape, fill, number)
- Keyboard and mouse input
- Visual feedback for valid/invalid sets (green/red card borders)
- Score tracking and deck counter
- **Hint**: highlights one card from a valid set
- **Auto-select**: finds and selects a valid set automatically
- **Deal extra**: deal extra cards when no SET exists on the board
- Clickable UI buttons in the info panel
- Dynamic board layout (rows scale with card count)
- Game over detection

## Controls

| Action | Keys |
|---|---|
| Move | Arrow keys / `W` `A` `S` `D` |
| Select card | `Enter` / `Space` |
| Hint | `H` |
| Auto-select | `X` |
| Deal extra | `E` |
| Quit | `Q` / `Esc` |
| Mouse | Click cards or buttons directly |

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2024 edition)

### Run

```sh
cargo run
```

### Build release

```sh
cargo build --release
```

## Project Structure

```
src/
├── main.rs    # Entry point, terminal setup/teardown
├── app.rs     # Application state and main loop
├── game.rs    # SET game logic, card types, validation
├── input.rs   # Keyboard and mouse event handling
├── ui.rs      # Board and info panel rendering
└── event.rs   # Terminal event polling
```