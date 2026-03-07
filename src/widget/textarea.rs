use crate::style::{BorderType, Style};
use crate::widget::Widget;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style as RatatuiStyle},
    widgets::{Block, Borders},
};
use ratatui_textarea::TextArea;
use std::sync::Mutex;

/// A multi-line text input widget.
pub struct Textarea {
    textarea: Mutex<TextArea<'static>>,
    border_type: Option<BorderType>,
    height: u16,
}

impl Textarea {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        // Multi-line mode (default)
        textarea.set_cursor_line_style(RatatuiStyle::default());
        textarea.set_cursor_style(RatatuiStyle::default().add_modifier(Modifier::REVERSED));

        Textarea {
            textarea: Mutex::new(textarea),
            border_type: None,
            height: 5, // Default height: 5 lines
        }
    }

    pub fn with_value<S: Into<String>>(value: S) -> Self {
        let mut textarea = TextArea::default();
        textarea.insert_str(&value.into());
        textarea.set_cursor_line_style(RatatuiStyle::default());
        textarea.set_cursor_style(RatatuiStyle::default().add_modifier(Modifier::REVERSED));

        Textarea {
            textarea: Mutex::new(textarea),
            border_type: None,
            height: 5,
        }
    }

    /// Create a textarea with border.
    pub fn bordered(mut self, border_type: BorderType) -> Self {
        self.border_type = Some(border_type);
        self
    }

    /// Set the height of the textarea (number of visible lines).
    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    pub fn value(&self) -> String {
        self.textarea.lock().unwrap().lines().join("\n")
    }

    pub fn set_value<S: Into<String>>(&mut self, value: S) {
        let mut textarea = TextArea::default();
        textarea.insert_str(&value.into());
        self.textarea = Mutex::new(textarea);
    }
}

impl Default for Textarea {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Textarea {
    fn render(&self, f: &mut Frame, area: Rect, style: &Style, is_focused: bool) {
        // Apply padding
        let inner_area = style.shrink(area);

        // Show border only if style.border_type is Some
        let block = if self.border_type.is_some() {
            Block::default()
                .borders(Borders::ALL)
                .border_type(match self.border_type {
                    Some(BorderType::Plain) => ratatui::widgets::BorderType::Plain,
                    Some(BorderType::Rounded) => ratatui::widgets::BorderType::Rounded,
                    Some(BorderType::Double) => ratatui::widgets::BorderType::Double,
                    Some(BorderType::Thick) => ratatui::widgets::BorderType::Thick,
                    None => ratatui::widgets::BorderType::Plain,
                })
        } else {
            Block::default()
        };

        // Get the inner area after border
        let value_area = block.inner(inner_area);

        // Render the block border
        f.render_widget(block, inner_area);

        // Set cursor style based on focus state
        let mut textarea = self.textarea.lock().unwrap();
        if is_focused {
            textarea.set_cursor_style(RatatuiStyle::default().add_modifier(Modifier::REVERSED));
        } else {
            textarea.set_cursor_style(RatatuiStyle::default());
        }

        // Render the textarea (with or without cursor based on focus)
        f.render_widget(&*textarea, value_area);
    }

    fn node_style_hint(&self) -> Option<Style> {
        // Return style based on border_type
        match self.border_type {
            Some(border_type) => Some(Style::new().border(border_type)),
            None => Some(Style::new().no_border()),
        }
    }

    fn content_size(&self, _area: Rect) -> (u16, u16) {
        // Textarea has fixed width of 20 characters and configured height
        let width = 20u16;
        let height = self.height;
        (width, height)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Enter key inserts newline in multi-line mode
        self.textarea.lock().unwrap().input(key);
        true
    }
}
