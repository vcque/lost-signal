use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Widget},
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
        };

        for (i, line) in help_text.iter().enumerate() {
            if i < inner.height as usize {
                line.render(
                    Rect {
                        x: inner.x,
                        y: inner.y + i as u16,
                        width: inner.width,
                        height: 1,
                    },
                    buf,
                );
            }
        }

        // Render page info in bottom right corner
        let page_y = popup_area.y + popup_area.height - 1;
        let page_x = popup_area.x + popup_area.width - page_info.len() as u16 - 1;
        buf.set_string(page_x, page_y, page_info, Style::default().fg(Color::Gray));
    }

    fn page_1(&self) -> Vec<Line<'_>> {
        vec![
            Line::from("Movement: Arrow Keys, Vi keys (hjklyubn), or Numpad (8246 + 7913)"),
            Line::from("Wait: 5 or Space |  Respawn: r  |  Help: ?"),
            Line::from("Sense Controls (Shift + Key): Up/Down=Select, Left/Right=Adjust"),
            Line::from(""),
            Line::from("Navigation: Use Left/Right arrow keys to change help pages"),
        ]
    }

    fn page_2(&self) -> Vec<Line<'_>> {
        vec![
            Line::from("Self: Monitor your integrity"),
            Line::from("Touch: Detect adjacent tiles and entities"),
            Line::from("Hearing: Detect orb distance (radius)"),
            Line::from("Sight: See nearby tiles and entities (radius)"),
        ]
    }

    fn page_3(&self) -> Vec<Line<'_>> {
        vec![
            Line::from("Each sense costs signal points per turn"),
            Line::from("Pylons restore your signal"),
            Line::from("Manage your signal budget carefully"),
            Line::from(""),
            Line::from("Higher sense levels cost more signal"),
        ]
    }

    fn page_4(&self) -> Vec<Line<'_>> {
        vec![
            Line::from("Find and reach the orb to win the game"),
            Line::from("Use your senses to navigate the world"),
            Line::from(""),
            Line::from("Progress through stages to advance"),
        ]
    }
}
