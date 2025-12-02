use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Widget, Wrap},
};

use crate::{
    tui::{THEME, utils::center},
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
            open: false,
            selection: 0,
            max: 10, // 4 pages (0-3)
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
                "Senses are your only way of gathering information on your surroundings. Enabled senses cost focus each turn and won't activate if you can't pay the cost.",
            ),
            Line::from(""),
            Line::from(
                "At the bottom of the lesser realities, only the (limited) sense of Self exists. But you might find more useful senses as you climb.",
            ),
            Line::from(""),
            Line::from("Controls: Shift + Left/Right to disable/enable a sense."),
            Line::from(""),
            Line::from("SELF SENSE - cost: 1"),
            Line::from("Shows your current focus level."),
            Line::from(""),
            Line::from("You somehow know in your inner Self that you must go north.").italic(),
        ])
    }

    fn page_2(&self) -> Paragraph<'_> {
        Paragraph::new(vec![
            Line::from("A tingling sensation overwhelms you as you reach a higher reality. You can feel again.").italic(),
            Line::from(""),
            Line::from("TOUCH SENSE - cost: 1"),
            Line::from("Shows the adjacent tiles."),
            Line::from("Shows how many adjacent entities there are."),
            Line::from(""),
            Line::from("FOCUS"),
            Line::from("You can recharge focus either by respawning with 'r' or by standing next to a pylon '|'."),
            Line::from(""),
            Line::from("SELECTING A SENSE"),
            Line::from("Shift + Up/Down to select a sense that you can then enable/disable."),
            Line::from(""),
            Line::from(vec![
                Span::from("NEW FOE: Mind Snare "), Span::from("¤").style(THEME.palette.foe).bold()]),
            Line::from("An immobile predator populating the most remote realities, where unsuspecting victims don't have the senses to easily detect them."),
            Line::from("It feeds on its victim Self, forcing them to witness it until they are no more."),
            Line::from(""),
            Line::from("THE ORB"),
            Line::from("The orb is a passageway between two realities. Grab it to climb further up."),
            Line::from(""),
            Line::from("Enable your touch sense to feel your surroundings and search for the orb."),
        ])
    }

    fn page_3(&self) -> Paragraph<'_> {
        Paragraph::new(vec![
            Line::from("Your steps echo through the corridors of this new reality as you ascend further. You can hear.").italic(),
            Line::from(""),
            Line::from("HEARING SENSE - cost: strength"),
            Line::from("Detects the distance to noise sources. The orb is a noise source."),
            Line::from("Strength (1, 2, 3, 4, 5) -> Distance in tiles: (3, 6, 10, 15, 21)"),
            Line::from(""),
            Line::from("STRENGTHENING SENSES"),
            Line::from("Some senses can be strengthened beyond their basic level."),
            Line::from("Shift + Left/Right to decrease/increase a sense's strength."),
            Line::from("Higher strength levels cost more focus but provide better information."),
            Line::from(""),
            Line::from("A FICKLE ORB"),
            Line::from("Beware: the orb is unpredictable and may sometimes relocate."),
        ])
    }

    fn page_4(&self) -> Paragraph<'_> {
        Paragraph::new(vec![
            Line::from("Light at last. The most powerful of the senses. You can see.").italic(),
            Line::from(""),
            Line::from("SIGHT SENSE - cost: 2 + strength"),
            Line::from("Shows your surrounding up to $strength distance."),
            Line::from("Shows the orb and foes in your field of view."),
            Line::from(""),
            Line::from("A SHY ORB"),
            Line::from(
                "The orb has a hard time bearing the gaze of others and will relocate if seen.",
            ),
        ])
    }
}
