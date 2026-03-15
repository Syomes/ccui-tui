use crate::event::{UiMessage, WidgetMessage};
use crate::style::Style;
use crate::widget::{Widget, WidgetKind, WidgetType};
use parking_lot::Mutex;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget as RatatuiWidget, Wrap},
};
use std::any::Any;
use std::sync::Arc;
use tokio::sync::mpsc;
use unicode_width::UnicodeWidthStr;

/// Messages for Text widget.
pub enum TextMessage {
    SetContent(String),
}

impl WidgetMessage for TextMessage {
    fn apply(self: Box<Self>, widget: &mut dyn Widget) {
        if let Some(text) = widget.as_any_mut().downcast_mut::<Text>() {
            match *self {
                TextMessage::SetContent(ref content) => {
                    *text.content.lock() = content.clone();
                }
            }
        }
    }
}

/// Handle for controlling a Text widget.
#[derive(Clone)]
pub struct TextHandle {
    id: String,
    style: crate::style::Style,
    ui_tx: mpsc::Sender<UiMessage>,
    content: Arc<Mutex<String>>,
}

impl crate::document::WidgetHandle for TextHandle {
    fn id(&self) -> &str {
        &self.id
    }
    fn style(&self) -> &crate::style::Style {
        &self.style
    }
    fn ui_tx(&self) -> &mpsc::Sender<UiMessage> {
        &self.ui_tx
    }
}

impl TextHandle {
    /// Get the current content.
    pub fn get_content(&self) -> String {
        self.content.lock().clone()
    }

    /// Set the content.
    pub fn set_content(
        &self,
        content: impl Into<String>,
    ) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.ui_tx.try_send(UiMessage::WidgetMessage {
            id: self.id.clone(),
            message: Box::new(TextMessage::SetContent(content.into())),
        })?;
        Ok(())
    }
}

impl WidgetType for Text {
    type Handle = TextHandle;

    fn kind() -> WidgetKind {
        WidgetKind::Text
    }

    fn create_handle(
        id: String,
        ui_tx: mpsc::Sender<UiMessage>,
        style: crate::style::Style,
    ) -> Self::Handle {
        let content = Arc::new(Mutex::new(String::new()));
        TextHandle {
            id,
            style,
            ui_tx,
            content: Arc::clone(&content),
        }
    }
}

/// A simple text display widget.
pub struct Text {
    content: Arc<Mutex<String>>,
}

impl Text {
    pub fn new<S: Into<String>>(content: S) -> Self {
        Text {
            content: Arc::new(Mutex::new(content.into())),
        }
    }

    pub fn content(&self) -> String {
        self.content.lock().clone()
    }

    pub fn set_content<S: Into<String>>(&mut self, content: S) {
        *self.content.lock() = content.into();
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new("")
    }
}

impl Widget for Text {
    fn render(&self, buffer: &mut Buffer, area: Rect, style: &Style, _is_focused: bool) {
        // Apply padding to get inner area
        let inner_area = style.shrink(area);

        // Text with automatic word wrapping based on available width
        let content = self.content.lock();
        let paragraph = Paragraph::new(content.as_str()).wrap(Wrap { trim: false });

        paragraph.render(inner_area, buffer);
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

        let content = self.content.lock();
        let mut total_lines = 0;
        let mut max_line_width = 0;

        for line in content.lines() {
            let line_width = line.width();
            max_line_width = max_line_width.max(line_width);

            // Calculate how many terminal lines this line needs
            let wrapped_lines = (line_width + max_width - 1) / max_width;
            total_lines += wrapped_lines.max(1);
        }

        (
            max_line_width.min(area.width as usize) as u16,
            total_lines as u16,
        )
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
