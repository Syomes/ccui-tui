use crate::style::Style;
use crate::widget::Widget;
use ratatui::{Frame, layout::Rect, widgets::Paragraph};
use unicode_width::UnicodeWidthStr;

/// A simple text display widget.
pub struct Text {
    content: String,
}

impl Text {
    pub fn new<S: Into<String>>(content: S) -> Self {
        Text {
            content: content.into(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_content<S: Into<String>>(&mut self, content: S) {
        self.content = content.into();
    }
}

impl Widget for Text {
    fn render(&self, f: &mut Frame, area: Rect, style: &Style) {
        // Apply padding to get inner area
        let inner_area = style.shrink(area);

        let paragraph = Paragraph::new(self.content.as_str());
        f.render_widget(paragraph, inner_area);
    }

    fn node_style_hint(&self) -> Option<Style> {
        Some(Style::new().column())
    }

    fn content_size(&self, area: Rect) -> (u16, u16) {
        // Text content size: unicode width of content * number of lines
        // Capped at available area
        let lines = self.content.lines().count() as u16;
        let max_line_len = self
            .content
            .lines()
            .map(|l| l.width() as u16)
            .max()
            .unwrap_or(0);
        (max_line_len.min(area.width), lines.min(area.height))
    }
}
