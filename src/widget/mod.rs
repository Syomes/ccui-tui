use crate::style::Style;
use crossterm::event::KeyEvent;
use ratatui::{buffer::Buffer, layout::Rect};
use tokio::sync::mpsc;

use crate::event::UiMessage;

/// Widget type identifier.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WidgetKind {
    Input,
    Textarea,
    Text,
    Divider,
}

/// Trait to associate a Widget with its Handle type.
pub trait WidgetType {
    type Handle: crate::document::WidgetHandle;

    fn kind() -> WidgetKind;

    fn create_handle(
        id: String,
        ui_tx: mpsc::Sender<UiMessage>,
        style: crate::style::Style,
    ) -> Self::Handle;
}

/// A renderable widget that can be displayed in a terminal area.
pub trait Widget: Send + Sync {
    /// Render the widget within the given buffer and area.
    fn render(&self, buffer: &mut Buffer, area: Rect, style: &Style, is_focused: bool);

    /// Get the default style hint for the node that contains this widget.
    fn node_style_hint(&self) -> Option<Style> {
        None
    }

    /// Get the preferred size hint for this widget.
    ///
    /// Returns `None` to use default behavior (fill available space).
    /// Returns `Some((width, height))` to request a specific size.
    /// - If width is 0, the widget will fill available width.
    /// - If height is 0, the widget will fill available height.
    ///
    /// Example: `Some((1, 0))` means 1 character wide, full height.
    fn size_hint(&self) -> Option<(u16, u16)> {
        None
    }

    /// Get the actual content size within the given area.
    /// Returns (width, height) of the actual content.
    /// Default: occupies the entire area.
    fn content_size(&self, area: Rect) -> (u16, u16) {
        (area.width, area.height)
    }

    /// Handle keyboard input when the widget has focus.
    /// Returns true if the event was handled, false otherwise.
    fn handle_key(&mut self, _key: KeyEvent) -> bool {
        false
    }

    /// Get mutable reference as Any for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub mod divider;
pub mod input;
pub mod text;
pub mod textarea;

pub use divider::{Direction, Divider, DividerHandle};
pub use input::{Input, InputHandle};
pub use text::{Text, TextHandle};
pub use textarea::{Textarea, TextareaHandle};
