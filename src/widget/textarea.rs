use crate::event::{UiMessage, WidgetMessage};
use crate::style::{BorderType, Style};
use crate::widget::{Widget, WidgetKind, WidgetType};
use crossterm::event::KeyEvent;
use parking_lot::Mutex;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style as RatatuiStyle},
    widgets::{Block, Borders, Widget as RatatuiWidget},
};
use ratatui_textarea::TextArea;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Messages for Textarea widget.
pub enum TextareaMessage {
    SetHeight(u16),
    SetValue(String),
}

impl WidgetMessage for TextareaMessage {
    fn apply(self: Box<Self>, widget: &mut dyn Widget) {
        if let Some(textarea) = widget.as_any_mut().downcast_mut::<Textarea>() {
            match *self {
                TextareaMessage::SetHeight(h) => textarea.height = h,
                TextareaMessage::SetValue(ref value) => {
                    let len = textarea.textarea.lock().lines().join("\n").len();
                    let mut ta = textarea.textarea.lock();
                    ta.delete_str(len);
                    ta.insert_str(value);
                }
            }
        }
    }
}

/// Handle for controlling a Textarea widget.
#[derive(Clone)]
pub struct TextareaHandle {
    id: String,
    style: crate::style::Style,
    ui_tx: mpsc::Sender<UiMessage>,
    textarea: Arc<Mutex<TextArea<'static>>>,
}

impl crate::document::WidgetHandle for TextareaHandle {
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

impl TextareaHandle {
    /// Get the current value.
    pub fn get_value(&self) -> String {
        self.textarea.lock().lines().join("\n")
    }

    /// Set the value.
    pub fn set_value(
        &self,
        value: impl Into<String>,
    ) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.ui_tx.try_send(UiMessage::WidgetMessage {
            id: self.id.clone(),
            message: Box::new(TextareaMessage::SetValue(value.into())),
        })?;
        Ok(())
    }

    /// Set the height of the textarea.
    pub fn set_height(&self, height: u16) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.ui_tx.try_send(UiMessage::WidgetMessage {
            id: self.id.clone(),
            message: Box::new(TextareaMessage::SetHeight(height)),
        })?;
        Ok(())
    }
}

impl WidgetType for Textarea {
    type Handle = TextareaHandle;

    fn kind() -> WidgetKind {
        WidgetKind::Textarea
    }

    fn create_handle(
        id: String,
        ui_tx: mpsc::Sender<UiMessage>,
        style: crate::style::Style,
    ) -> Self::Handle {
        let textarea = Arc::new(Mutex::new(TextArea::default()));
        TextareaHandle {
            id,
            style,
            ui_tx,
            textarea: Arc::clone(&textarea),
        }
    }
}

/// A multi-line text input widget.
pub struct Textarea {
    textarea: Arc<Mutex<TextArea<'static>>>,
    border_type: Option<BorderType>,
    height: u16,
}

impl Textarea {
    pub fn new() -> Self {
        let textarea = Arc::new(Mutex::new({
            let mut ta = TextArea::default();
            ta.set_cursor_line_style(RatatuiStyle::default());
            ta.set_cursor_style(RatatuiStyle::default().add_modifier(Modifier::REVERSED));
            ta
        }));

        Textarea {
            textarea,
            border_type: None,
            height: 5,
        }
    }

    pub fn with_value<S: Into<String>>(value: S) -> Self {
        let textarea = Arc::new(Mutex::new({
            let mut ta = TextArea::default();
            ta.insert_str(&value.into());
            ta.set_cursor_line_style(RatatuiStyle::default());
            ta.set_cursor_style(RatatuiStyle::default().add_modifier(Modifier::REVERSED));
            ta
        }));

        Textarea {
            textarea,
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
        self.textarea.lock().lines().join("\n")
    }

    pub fn set_value<S: Into<String>>(&mut self, value: S) {
        let mut ta = TextArea::default();
        ta.insert_str(&value.into());
        *self.textarea.lock() = ta;
    }
}

impl Default for Textarea {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Textarea {
    fn render(&self, buffer: &mut Buffer, area: Rect, style: &Style, is_focused: bool) {
        // Apply padding
        let inner_area = style.shrink(area);

        // Show border only if needed
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
        block.render(inner_area, buffer);

        // Set cursor style based on focus state
        let mut textarea = self.textarea.lock();
        if is_focused {
            textarea.set_cursor_style(RatatuiStyle::default().add_modifier(Modifier::REVERSED));
        } else {
            textarea.set_cursor_style(RatatuiStyle::default());
        }

        // Render the textarea (with or without cursor based on focus)
        textarea.render(value_area, buffer);
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
        self.textarea.lock().input(key);
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
