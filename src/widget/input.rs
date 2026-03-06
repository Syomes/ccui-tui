use crate::style::{BorderType, Style};
use crate::widget::Widget;
use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};
use tui_input::Input as TuiInput;
use tui_input::backend::crossterm::EventHandler;

/// An input field widget that accepts text input.
pub struct Input {
    input: TuiInput,
    border_type: Option<BorderType>,
}

impl Input {
    pub fn new() -> Self {
        Input {
            input: TuiInput::default(),
            border_type: None, // Default: no border
        }
    }

    pub fn with_value<S: Into<String>>(value: S) -> Self {
        Input {
            input: TuiInput::default().with_value(value.into()),
            border_type: None,
        }
    }

    /// Create an input with border.
    pub fn bordered(mut self, border_type: BorderType) -> Self {
        self.border_type = Some(border_type);
        self
    }

    pub fn value(&self) -> &str {
        self.input.value()
    }

    pub fn set_value<S: Into<String>>(&mut self, value: S) {
        self.input = TuiInput::default().with_value(value.into());
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Input {
    fn render(&self, f: &mut Frame, area: Rect, style: &Style, _is_focused: bool) {
        // Apply padding
        let inner_area = style.shrink(area);

        // Show border only if style.border_type is Some
        let block = if style.border_type.is_some() {
            Block::default()
                .borders(Borders::ALL)
                .border_type(match style.border_type {
                    Some(BorderType::Plain) => ratatui::widgets::BorderType::Plain,
                    Some(BorderType::Rounded) => ratatui::widgets::BorderType::Rounded,
                    Some(BorderType::Double) => ratatui::widgets::BorderType::Double,
                    Some(BorderType::Thick) => ratatui::widgets::BorderType::Thick,
                    None => ratatui::widgets::BorderType::Plain,
                })
        } else {
            Block::default()
        };

        // Render input
        let paragraph = Paragraph::new(self.input.value()).block(block);

        f.render_widget(paragraph, inner_area);
    }

    fn node_style_hint(&self) -> Option<Style> {
        // Return style based on border_type
        match self.border_type {
            Some(border_type) => Some(Style::new().border(border_type)),
            None => Some(Style::new().no_border()),
        }
    }

    fn content_size(&self, _area: Rect) -> (u16, u16) {
        // Input width: content length, height: 1 for text
        let width = self.input.value().len() as u16;
        let height = 1u16;
        (width, height)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.input.handle_event(&CrosstermEvent::Key(key)).is_some()
    }

    fn render_cursor(&self, f: &mut Frame, area: Rect, style: &Style, is_focused: bool) {
        if is_focused {
            // Calculate inner area (after padding)
            let inner_area = style.shrink(area);

            // Border offset (1px if border is shown)
            let border_offset = if style.border_type.is_some() { 1 } else { 0 };

            let cursor_x = inner_area.left() + border_offset + (self.input.visual_cursor() as u16);
            let cursor_y = inner_area.top() + border_offset;

            f.set_cursor_position((cursor_x, cursor_y));
        }
    }
}
