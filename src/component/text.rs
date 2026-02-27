use ratatui::{Frame, layout::Rect, widgets::Paragraph};
use crate::component::Component;
use crate::style::Style;

/// A simple text display component.
pub struct Text {
    content: String,
    style: Style,
}

impl Text {
    pub fn new<S: Into<String>>(content: S) -> Self {
        Text {
            content: content.into(),
            style: Style::new(),
        }
    }
    
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    
    pub fn get_style(&self) -> &Style {
        &self.style
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_content<S: Into<String>>(&mut self, content: S) {
        self.content = content.into();
    }
}

impl Component for Text {
    fn render(&self, f: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(self.content.as_str());
        f.render_widget(paragraph, area);
    }
}
