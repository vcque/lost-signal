use std::{cell::RefCell, io::Result, rc::Rc};

use log::error;
use ratatui::Terminal;
use ratzilla::{WebGl2Backend, WebRenderer, event as rz};

use losig_client::{
    adapter::TuiAdapter,
    tui_adapter::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, TuiApp},
};

pub struct RatzillaAdapter {}

impl RatzillaAdapter {
    pub fn new() -> Self {
        RatzillaAdapter {}
    }
    fn run_inner<T: TuiApp + 'static>(self, app: T) -> Result<()> {
        let backend = WebGl2Backend::new()?;
        let terminal = Terminal::new(backend)?;

        let app = Rc::new(RefCell::new(app));
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

impl TuiAdapter for RatzillaAdapter {
    fn run<T: TuiApp + 'static>(self, app: T) {
        if let Err(e) = self.run_inner(app) {
            error!("Couldn't start tui: {e}");
        }
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
