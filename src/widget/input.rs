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

        // Get the inner area after border
        let value_area = block.inner(inner_area);
        // Reserve 1 character for cursor display
        let render_width = value_area.width.saturating_sub(1) as usize;

        // Calculate scroll offset for rendering
        let scroll_offset = self.input.visual_scroll(render_width);

        // Get visible text
        let visible_text = &self.input.value()[scroll_offset..];

        // Truncate to fit width
        let visible_text: String = visible_text.chars().take(render_width).collect();

        // Render the visible portion
        let paragraph = Paragraph::new(visible_text.as_str()).block(block);
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
        // Input has a fixed width of 10 characters
        // Height is always 1 for single-line input
        let width = 10u16;
        let height = 1u16;
        (width, height)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.input.handle_event(&CrosstermEvent::Key(key)).is_some()
    }

    fn render_cursor(&self, f: &mut Frame, area: Rect, style: &Style, is_focused: bool) {
        if is_focused {
            // Calculate inner area (after padding and border)
            let inner_area = style.shrink(area);
            let block = if style.border_type.is_some() {
                Block::default().borders(Borders::ALL)
            } else {
                Block::default()
            };
            let value_area = block.inner(inner_area);

            // Same as render: reserve 1 character for cursor
            let render_width = value_area.width.saturating_sub(1) as usize;

            // Calculate scroll offset (same as render)
            let scroll_offset = self.input.visual_scroll(render_width);

            // Ensure cursor is always within the rendered area
            let cursor_x = value_area.left() + ((self.input.cursor() - scroll_offset) as u16);
            let cursor_y = value_area.top();

            f.set_cursor_position((cursor_x, cursor_y));
        }
    }
}
