use std::{
    cell::RefCell,
    io::{self},
    rc::Rc,
};

use ratatui::Terminal;
use ratzilla::{CanvasBackend, WebRenderer, event as rz};

use losig_client::tui_adapter::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind, TuiApp,
};

pub struct RatzillaAdapter<T> {
    app: T,
}

impl<T: TuiApp + 'static> RatzillaAdapter<T> {
    pub fn new(app: T) -> RatzillaAdapter<T> {
        RatzillaAdapter { app }
    }

    pub fn run(self) -> io::Result<()> {
        let backend = CanvasBackend::new()?;
        let terminal = Terminal::new(backend)?;

        let app = Rc::new(RefCell::new(self.app));

        let event_app = Rc::clone(&app);

        terminal.on_key_event(move |key_event| {
            let adapter_event = Event::Key(convert_key_event(key_event));
            event_app.borrow_mut().handle_events(adapter_event);
        });

        let event_app = Rc::clone(&app);
        terminal.draw_web(move |frame| {
            event_app.borrow_mut().render(frame);
        });

        Ok(())
    }
}

// Conversion functions for Ratzilla events to adapter events
pub fn convert_key_event(rz_key: rz::KeyEvent) -> KeyEvent {
    KeyEvent {
        code: convert_key_code(rz_key.code),
        modifiers: convert_key_modifiers(rz_key.shift, rz_key.ctrl, rz_key.alt),
        kind: KeyEventKind::Press,
        state: KeyEventState::None,
    }
}

fn convert_key_code(rz_code: rz::KeyCode) -> KeyCode {
    match rz_code {
        rz::KeyCode::Backspace => KeyCode::Backspace,
        rz::KeyCode::Enter => KeyCode::Enter,
        rz::KeyCode::Left => KeyCode::Left,
        rz::KeyCode::Right => KeyCode::Right,
        rz::KeyCode::Up => KeyCode::Up,
        rz::KeyCode::Down => KeyCode::Down,
        rz::KeyCode::Home => KeyCode::Home,
        rz::KeyCode::End => KeyCode::End,
        rz::KeyCode::PageUp => KeyCode::PageUp,
        rz::KeyCode::PageDown => KeyCode::PageDown,
        rz::KeyCode::Tab => KeyCode::Tab,
        rz::KeyCode::Delete => KeyCode::Delete,
        rz::KeyCode::F(n) => KeyCode::F(n),
        rz::KeyCode::Char(c) => KeyCode::Char(c),
        rz::KeyCode::Esc => KeyCode::Esc,
        rz::KeyCode::Unidentified => KeyCode::Null,
    }
}
pub fn convert_mouse_event(rz_mouse: rz::MouseEvent) -> MouseEvent {
    MouseEvent {
        kind: convert_mouse_event_kind(rz_mouse.event, rz_mouse.button),
        column: rz_mouse.x as u16,
        row: rz_mouse.y as u16,
        modifiers: convert_key_modifiers(rz_mouse.shift, rz_mouse.ctrl, rz_mouse.alt),
    }
}

fn convert_mouse_event_kind(
    rz_kind: rz::MouseEventKind,
    rz_button: rz::MouseButton,
) -> MouseEventKind {
    match rz_kind {
        rz::MouseEventKind::Pressed => MouseEventKind::Down(convert_mouse_button(rz_button)),
        rz::MouseEventKind::Released => MouseEventKind::Up(convert_mouse_button(rz_button)),
        rz::MouseEventKind::Moved => MouseEventKind::Moved,
        rz::MouseEventKind::Unidentified => MouseEventKind::Moved,
    }
}

fn convert_mouse_button(rz_button: rz::MouseButton) -> MouseButton {
    match rz_button {
        rz::MouseButton::Left => MouseButton::Left,
        rz::MouseButton::Right => MouseButton::Right,
        rz::MouseButton::Middle => MouseButton::Middle,
        rz::MouseButton::Forward => MouseButton::Left,
        rz::MouseButton::Back => MouseButton::Right,
        rz::MouseButton::Unidentified => MouseButton::Middle,
    }
}

fn convert_key_modifiers(shift: bool, control: bool, alt: bool) -> KeyModifiers {
    KeyModifiers {
        shift,
        control,
        alt,
        super_key: false,
        hyper: false,
        meta: false,
    }
}
