use crate::actions::Action;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

/// Non-blocking input poll.  Returns `Some(Action)` if a recognised key
/// was pressed this tick, `None` otherwise.  Consumes (and discards)
/// resize and other non-key events so they don't queue up.
pub fn poll_action() -> Option<Action> {
    if !event::poll(Duration::ZERO).ok()? {
        return None;
    }
    match event::read().ok()? {
        Event::Key(key) => key_to_action(key),
        _ => None, // resize, mouse, etc. — consumed and ignored
    }
}

fn key_to_action(key: KeyEvent) -> Option<Action> {
    // On Windows crossterm sends Press, Repeat, and Release events.
    // Only act on Press to avoid double-firing.
    if key.kind != KeyEventKind::Press {
        return None;
    }

    match key.code {
        // Quit
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Esc => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Quit),

        // Speed
        KeyCode::Char('+') | KeyCode::Char('=') => Some(Action::IncreaseSpeed),
        KeyCode::Char('-') => Some(Action::DecreaseSpeed),

        // Density
        KeyCode::Char('[') => Some(Action::DecreaseDensity),
        KeyCode::Char(']') => Some(Action::IncreaseDensity),

        // Palette / wind / warp
        KeyCode::Char('c') => Some(Action::CyclePalette),
        KeyCode::Char('w') => Some(Action::CycleWind),
        KeyCode::Char('g') => Some(Action::ToggleWarp),

        // Reseed / storm
        KeyCode::Char('r') => Some(Action::Reseed),
        KeyCode::Char('p') => Some(Action::ToggleStorm),

        _ => None,
    }
}
