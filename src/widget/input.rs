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

/// Messages for Input widget.
pub enum InputMessage {
    SetMask(Option<char>),
    SetValue(String),
}

impl WidgetMessage for InputMessage {
    fn apply(self: Box<Self>, widget: &mut dyn Widget) {
        if let Some(input) = widget.as_any_mut().downcast_mut::<Input>() {
            match *self {
                InputMessage::SetMask(ch) => input.mask_char = ch,
                InputMessage::SetValue(ref value) => {
                    let len = input.textarea.lock().lines().join("\n").len();
                    let mut ta = input.textarea.lock();
                    ta.delete_str(len);
                    ta.insert_str(value);
                }
            }
        }
    }
}

/// Handle for controlling an Input widget.
#[derive(Clone)]
pub struct InputHandle {
    id: String,
    style: crate::style::Style,
    ui_tx: mpsc::Sender<UiMessage>,
    textarea: Arc<Mutex<TextArea<'static>>>,
}

impl crate::document::WidgetHandle for InputHandle {
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

impl InputHandle {
    /// Get the current value.
    pub fn get_value(&self) -> String {
        self.textarea.lock().lines().join("\n")
    }

    /// Set the value.
    pub fn set_value(
        &self,
        value: impl Into<String>,
    ) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        // Update via message
        self.ui_tx.try_send(UiMessage::WidgetMessage {
            id: self.id.clone(),
            message: Box::new(InputMessage::SetValue(value.into())),
        })?;
        Ok(())
    }

    /// Set the mask character for password input.
    pub fn set_masked(&self, ch: Option<char>) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.ui_tx.try_send(UiMessage::WidgetMessage {
            id: self.id.clone(),
            message: Box::new(InputMessage::SetMask(ch)),
        })?;
        Ok(())
    }

    /// Enable masking with default character '*'.
    pub fn masked(&self) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.set_masked(Some('*'))
    }

    /// Enable masking with custom character.
    pub fn masked_with(&self, ch: char) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.set_masked(Some(ch))
    }

    /// Disable masking.
    pub fn unmasked(&self) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.set_masked(None)
    }
}

impl WidgetType for Input {
    type Handle = InputHandle;

    fn kind() -> WidgetKind {
        WidgetKind::Input
    }

    fn create_handle(
        id: String,
        ui_tx: mpsc::Sender<UiMessage>,
        style: crate::style::Style,
    ) -> Self::Handle {
        let textarea = Arc::new(Mutex::new(TextArea::default()));
        InputHandle {
            id,
            style,
            ui_tx,
            textarea: Arc::clone(&textarea),
        }
    }
}

/// An input field widget that accepts text input (single-line mode).
pub struct Input {
    textarea: Arc<Mutex<TextArea<'static>>>,
    border_type: Option<BorderType>,
    mask_char: Option<char>,
}

impl Input {
    pub fn new() -> Self {
        let textarea = Arc::new(Mutex::new({
            let mut ta = TextArea::default();
            ta.set_cursor_line_style(RatatuiStyle::default());
            ta.set_cursor_style(RatatuiStyle::default());
            ta
        }));

        Input {
            textarea,
            border_type: None,
            mask_char: None,
        }
    }

    pub fn with_value<S: Into<String>>(value: S) -> Self {
        let textarea = Arc::new(Mutex::new({
            let mut ta = TextArea::default();
            ta.insert_str(&value.into());
            ta.set_cursor_line_style(RatatuiStyle::default());
            ta.set_cursor_style(RatatuiStyle::default());
            ta
        }));

        Input {
            textarea,
            border_type: None,
            mask_char: None,
        }
    }

    /// Mask input with default character '*'.
    pub fn masked(self) -> Self {
        self.masked_with('*')
    }

    /// Mask input with custom character.
    pub fn masked_with(mut self, ch: char) -> Self {
        self.mask_char = Some(ch);
        self
    }

    /// Disable masking.
    pub fn unmasked(mut self) -> Self {
        self.mask_char = None;
        self
    }

    /// Create an input with border.
    pub fn bordered(mut self, border_type: BorderType) -> Self {
        self.border_type = Some(border_type);
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

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Input {
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

        // Set mask character if needed
        if let Some(ch) = self.mask_char {
            textarea.set_mask_char(ch);
        } else {
            textarea.clear_mask_char();
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
        // Input has a fixed width of 10 characters
        // Height is always 1 for single-line input
        let width = 10u16;
        let height = 1u16;
        (width, height)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Ignore Enter key in single-line mode
        use crossterm::event::KeyCode;
        if key.code == KeyCode::Enter {
            return false;
        }
        self.textarea.lock().input(key);
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
