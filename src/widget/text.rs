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

        // Text with automatic word wrapping based on available width
        let paragraph = Paragraph::new(self.content.as_str())
            .wrap(ratatui::widgets::Wrap { trim: false });
        
        f.render_widget(paragraph, inner_area);
    }

    fn node_style_hint(&self) -> Option<Style> {
        Some(Style::new().column())
    }

    fn content_size(&self, area: Rect) -> (u16, u16) {
        // Text content size with word wrapping based on available width
        let max_width = area.width as usize;
        if max_width == 0 {
            return (0, 0);
        }
        
        let mut total_lines = 0;
        let mut max_line_width = 0;
        
        for line in self.content.lines() {
            let line_width = line.width();
            max_line_width = max_line_width.max(line_width);
            
            // Calculate how many terminal lines this line needs
            let wrapped_lines = (line_width + max_width - 1) / max_width;
            total_lines += wrapped_lines.max(1);
        }
        
        (max_line_width.min(area.width as usize) as u16, total_lines as u16)
    }
}
