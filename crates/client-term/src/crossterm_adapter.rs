use std::{io::Stdout, time::Duration};

use anyhow::Result;
use crossterm::event as ct;
use losig_client::tui_adapter::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MediaKeyCode,
    ModifierKeyCode, MouseButton, MouseEvent, MouseEventKind, TuiApp,
};
use ratatui::{Terminal, prelude::CrosstermBackend};

pub struct CrosstermAdapter<T: TuiApp> {
    app: T,
}

impl<T: TuiApp> CrosstermAdapter<T> {
    pub fn new(app: T) -> CrosstermAdapter<T> {
        CrosstermAdapter { app }
    }

    pub fn run(self) {
        let mut terminal = ratatui::init();
        let result = self.do_run(&mut terminal);
        ratatui::restore();
        println!("Tui ended: {result:?}");
    }

    pub fn do_run(mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.app.render(f))?;
            if ct::poll(Duration::from_millis(50))? {
                let event = ct::read()?;

                // Check for Ctrl+C to exit
                if let ct::Event::Key(key) = &event {
                    if key.code == ct::KeyCode::Char('c')
                        && key.modifiers.contains(ct::KeyModifiers::CONTROL)
                    {
                        break;
                    }
                }

                self.app.handle_events(convert_event(event));
            }
        }
        Ok(())
    }
}

pub fn convert_event(ct_event: ct::Event) -> Event {
    match ct_event {
        ct::Event::FocusGained => Event::FocusGained,
        ct::Event::FocusLost => Event::FocusLost,
        ct::Event::Key(key_event) => Event::Key(convert_key_event(key_event)),
        ct::Event::Mouse(mouse_event) => Event::Mouse(convert_mouse_event(mouse_event)),
        ct::Event::Paste(text) => Event::Paste(text),
        ct::Event::Resize(cols, rows) => Event::Resize(cols, rows),
    }
}

fn convert_key_event(ct_key: ct::KeyEvent) -> KeyEvent {
    KeyEvent {
        code: convert_key_code(ct_key.code),
        modifiers: convert_key_modifiers(ct_key.modifiers),
        kind: convert_key_event_kind(ct_key.kind),
        state: convert_key_event_state(ct_key.state),
    }
}

fn convert_key_code(ct_code: ct::KeyCode) -> KeyCode {
    match ct_code {
        ct::KeyCode::Backspace => KeyCode::Backspace,
        ct::KeyCode::Enter => KeyCode::Enter,
        ct::KeyCode::Left => KeyCode::Left,
        ct::KeyCode::Right => KeyCode::Right,
        ct::KeyCode::Up => KeyCode::Up,
        ct::KeyCode::Down => KeyCode::Down,
        ct::KeyCode::Home => KeyCode::Home,
        ct::KeyCode::End => KeyCode::End,
        ct::KeyCode::PageUp => KeyCode::PageUp,
        ct::KeyCode::PageDown => KeyCode::PageDown,
        ct::KeyCode::Tab => KeyCode::Tab,
        ct::KeyCode::BackTab => KeyCode::BackTab,
        ct::KeyCode::Delete => KeyCode::Delete,
        ct::KeyCode::Insert => KeyCode::Insert,
        ct::KeyCode::F(n) => KeyCode::F(n),
        ct::KeyCode::Char(c) => KeyCode::Char(c),
        ct::KeyCode::Null => KeyCode::Null,
        ct::KeyCode::Esc => KeyCode::Esc,
        ct::KeyCode::CapsLock => KeyCode::CapsLock,
        ct::KeyCode::ScrollLock => KeyCode::ScrollLock,
        ct::KeyCode::NumLock => KeyCode::NumLock,
        ct::KeyCode::PrintScreen => KeyCode::PrintScreen,
        ct::KeyCode::Pause => KeyCode::Pause,
        ct::KeyCode::Menu => KeyCode::Menu,
        ct::KeyCode::KeypadBegin => KeyCode::KeypadBegin,
        ct::KeyCode::Media(media) => KeyCode::Media(convert_media_key_code(media)),
        ct::KeyCode::Modifier(modifier) => KeyCode::Modifier(convert_modifier_key_code(modifier)),
    }
}

fn convert_media_key_code(ct_media: ct::MediaKeyCode) -> MediaKeyCode {
    match ct_media {
        ct::MediaKeyCode::Play => MediaKeyCode::Play,
        ct::MediaKeyCode::Pause => MediaKeyCode::Pause,
        ct::MediaKeyCode::PlayPause => MediaKeyCode::PlayPause,
        ct::MediaKeyCode::Reverse => MediaKeyCode::Reverse,
        ct::MediaKeyCode::Stop => MediaKeyCode::Stop,
        ct::MediaKeyCode::FastForward => MediaKeyCode::FastForward,
        ct::MediaKeyCode::Rewind => MediaKeyCode::Rewind,
        ct::MediaKeyCode::TrackNext => MediaKeyCode::TrackNext,
        ct::MediaKeyCode::TrackPrevious => MediaKeyCode::TrackPrevious,
        ct::MediaKeyCode::Record => MediaKeyCode::Record,
        ct::MediaKeyCode::LowerVolume => MediaKeyCode::LowerVolume,
        ct::MediaKeyCode::RaiseVolume => MediaKeyCode::RaiseVolume,
        ct::MediaKeyCode::MuteVolume => MediaKeyCode::MuteVolume,
    }
}

fn convert_modifier_key_code(ct_modifier: ct::ModifierKeyCode) -> ModifierKeyCode {
    match ct_modifier {
        ct::ModifierKeyCode::LeftShift => ModifierKeyCode::LeftShift,
        ct::ModifierKeyCode::LeftControl => ModifierKeyCode::LeftControl,
        ct::ModifierKeyCode::LeftAlt => ModifierKeyCode::LeftAlt,
        ct::ModifierKeyCode::LeftSuper => ModifierKeyCode::LeftSuper,
        ct::ModifierKeyCode::LeftHyper => ModifierKeyCode::LeftHyper,
        ct::ModifierKeyCode::LeftMeta => ModifierKeyCode::LeftMeta,
        ct::ModifierKeyCode::RightShift => ModifierKeyCode::RightShift,
        ct::ModifierKeyCode::RightControl => ModifierKeyCode::RightControl,
        ct::ModifierKeyCode::RightAlt => ModifierKeyCode::RightAlt,
        ct::ModifierKeyCode::RightSuper => ModifierKeyCode::RightSuper,
        ct::ModifierKeyCode::RightHyper => ModifierKeyCode::RightHyper,
        ct::ModifierKeyCode::RightMeta => ModifierKeyCode::RightMeta,
        ct::ModifierKeyCode::IsoLevel3Shift => ModifierKeyCode::IsoLevel3Shift,
        ct::ModifierKeyCode::IsoLevel5Shift => ModifierKeyCode::IsoLevel5Shift,
    }
}

fn convert_key_modifiers(ct_modifiers: ct::KeyModifiers) -> KeyModifiers {
    KeyModifiers {
        shift: ct_modifiers.contains(ct::KeyModifiers::SHIFT),
        control: ct_modifiers.contains(ct::KeyModifiers::CONTROL),
        alt: ct_modifiers.contains(ct::KeyModifiers::ALT),
        super_key: ct_modifiers.contains(ct::KeyModifiers::SUPER),
        hyper: ct_modifiers.contains(ct::KeyModifiers::HYPER),
        meta: ct_modifiers.contains(ct::KeyModifiers::META),
    }
}

fn convert_key_event_kind(ct_kind: ct::KeyEventKind) -> KeyEventKind {
    match ct_kind {
        ct::KeyEventKind::Press => KeyEventKind::Press,
        ct::KeyEventKind::Repeat => KeyEventKind::Repeat,
        ct::KeyEventKind::Release => KeyEventKind::Release,
    }
}

fn convert_key_event_state(ct_state: ct::KeyEventState) -> KeyEventState {
    match ct_state {
        ct::KeyEventState::NONE => KeyEventState::None,
        ct::KeyEventState::KEYPAD => KeyEventState::Keypad,
        ct::KeyEventState::CAPS_LOCK => KeyEventState::CapsLock,
        ct::KeyEventState::NUM_LOCK => KeyEventState::NumLock,
        _ => KeyEventState::None, // Handle any other flags as None
    }
}

fn convert_mouse_event(ct_mouse: ct::MouseEvent) -> MouseEvent {
    MouseEvent {
        kind: convert_mouse_event_kind(ct_mouse.kind),
        column: ct_mouse.column,
        row: ct_mouse.row,
        modifiers: convert_key_modifiers(ct_mouse.modifiers),
    }
}

fn convert_mouse_event_kind(ct_kind: ct::MouseEventKind) -> MouseEventKind {
    match ct_kind {
        ct::MouseEventKind::Down(button) => MouseEventKind::Down(convert_mouse_button(button)),
        ct::MouseEventKind::Up(button) => MouseEventKind::Up(convert_mouse_button(button)),
        ct::MouseEventKind::Drag(button) => MouseEventKind::Drag(convert_mouse_button(button)),
        ct::MouseEventKind::Moved => MouseEventKind::Moved,
        ct::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
        ct::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
        ct::MouseEventKind::ScrollLeft => MouseEventKind::ScrollLeft,
        ct::MouseEventKind::ScrollRight => MouseEventKind::ScrollRight,
    }
}

fn convert_mouse_button(ct_button: ct::MouseButton) -> MouseButton {
    match ct_button {
        ct::MouseButton::Left => MouseButton::Left,
        ct::MouseButton::Right => MouseButton::Right,
        ct::MouseButton::Middle => MouseButton::Middle,
    }
}
