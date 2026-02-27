use crate::style::Style;
use crate::widget::Widget;
use ratatui::{Frame, layout::Rect, widgets::Paragraph};

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
}
