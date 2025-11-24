use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};

use crate::{
    tui::utils::center,
    tui_adapter::{Event, KeyCode},
};

pub struct HelpWidget;

#[derive(Debug)]
pub struct HelpState {
    pub open: bool,
    pub selection: u8,
    /// max help screen that can be shown
    pub max: u8,
}
impl HelpState {
    pub fn next_page(&mut self, page: u8) {
        let next_page = page.min(3);
        if next_page > self.max {
            self.max = next_page;
            self.selection = next_page;
            self.open = true;
        }
    }
}

impl Default for HelpState {
    fn default() -> Self {
        Self {
            open: true,
            selection: 0,
            max: 0, // 4 pages (0-3)
        }
    }
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
        let popup_area = center(area, Constraint::Percentage(50), Constraint::Percentage(50));

        // Clear the popup area with a background
        for x in popup_area.x..popup_area.x + popup_area.width {
            for y in popup_area.y..popup_area.y + popup_area.height {
                buf.set_string(x, y, " ", Style::default().bg(Color::Black));
            }
        }

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
            1 => self.page_2(),
            2 => self.page_3(),
            3 => self.page_4(),
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
            Line::from("Welcome to LOSIG!").alignment(Alignment::Center),
            Line::from(""),
            Line::from(
                "LOSIG is a (WIP) game about perception. You play as someone lost at the bottom of the lesser realities and need to reach the surface to become whole again.",
            ),
            Line::from(""),
            Line::from("CONTROLS"),
            Line::from("Movement: Arrow Keys, Vi keys (hjklyubn), or Numpad"),
            Line::from("Wait: 5 or Space | Help: ?"),
            Line::from(""),
            Line::from("SENSES"),
            Line::from(
                "Senses are your only way of gathering information on your surroundings. Enabled senses cost focus each turn and don't activate if you can't pay the cost.",
            ),
            Line::from(""),
            Line::from(
                "At the bottom of the lesser realities, only the (limited) sense of Self exists. But you might find more useful senses as you climb.",
            ),
            Line::from(""),
            Line::from("Controls: Shift + Left/Right to disable/enable a sense"),
            Line::from(""),
            Line::from("SELF SENSE - cost: 1"),
            Line::from("Shows your current focus level."),
            Line::from(""),
            Line::from("You somehow know in your inner Self that you must go north.").italic(),
        ])
    }

    fn page_2(&self) -> Paragraph<'_> {
        Paragraph::new(vec![])
    }

    fn page_3(&self) -> Paragraph<'_> {
        Paragraph::new(vec![])
    }

    fn page_4(&self) -> Paragraph<'_> {
        Paragraph::new(vec![])
    }
}
