use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Padding, Paragraph, Widget, Wrap},
};

use crate::{
    tui::utils::center,
    tui_adapter::{Event, KeyCode},
};

pub struct HelpWidget;

#[derive(Default, Debug)]
pub struct HelpState {
    pub open: bool,
    pub selection: u8,
    /// max help screen that can be shown
    pub max: u8,
}

impl HelpWidget {
    pub fn on_event(&self, event: &Event, state: &mut HelpState) -> bool {
        if !state.open {
            return false;
        }

        let Event::Key(key) = event else {
            return true; // Consume all non-key events when help is visible
        };

        match key.code {
            KeyCode::Char('?') | KeyCode::Esc => {
                state.open = false;
            }
            KeyCode::Left => {
                state.selection = state.selection.saturating_sub(1);
            }
            KeyCode::Right => {
                state.selection = (state.selection + 1).min(state.max);
            }
            _ => {} // Consume all other key events when help is visible
        }
        // always preempt other bindings if open
        true
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, state: &HelpState) {
        let popup_area = center(area, Constraint::Percentage(60), Constraint::Percentage(60));

        // Clear the popup area to reset style
        Clear.render(popup_area, buf);

        let title = "Help - Press '?' or 'ESC' to close";
        let page_info = format!("< page {} of {} >", state.selection + 1, state.max + 1);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let help_text = match state.selection {
            0 => self.page_1(),
            _ => self.page_1(),
        }
        .wrap(Wrap { trim: true });

        help_text
            .block(Block::new().padding(Padding::uniform(1)))
            .render(inner, buf);

        // Render page info in bottom right corner
        let page_y = popup_area.y + popup_area.height - 1;
        let page_x = popup_area.x + popup_area.width - page_info.len() as u16 - 1;
        buf.set_string(page_x, page_y, page_info, Style::default().fg(Color::Gray));
    }

    fn page_1(&self) -> Paragraph<'_> {
        Paragraph::new(vec![
            Line::from(""),
            Line::from("CONTROLS"),
            Line::from("Movement: Arrow Keys, Vi keys (hjklyubn), or Numpad"),
            Line::from("Wait: 5 or Space | Help: ?"),
            Line::from(""),
            Line::from("SENSE CONTROLS"),
            Line::from("Sense selection: Shift + Up/Down"),
            Line::from("Weaken/Strengthen sense: Shift + Left/Right"),
            Line::from(""),
            Line::from("SELF SENSE - cost: 1"),
            Line::from("Shows your current hp and focus level."),
            Line::from(""),
            Line::from("TOUCH SENSE - cost: 1"),
            Line::from("Shows the adjacent terrain/traps/enemies."),
            Line::from(""),
            Line::from("HEARING SENSE - cost: STRENGTH"),
            Line::from("Shows the sound sources and their approximate distance in tiles."),
            Line::from("The higher the STRENGTH the higher the range."),
            Line::from(""),
            Line::from("SIGHT SENSE - cost: 2 + STRENGTH"),
            Line::from("Shows enemies, players and terrain in a STRENGTH radius."),
        ])
    }
}
