use crate::style::Style;
use ratatui::{Frame, layout::Rect};

/// A renderable widget that can be displayed in a terminal area.
pub trait Widget: Send + Sync {
    /// Render the widget within the given frame and area.
    fn render(&self, f: &mut Frame, area: Rect, style: &Style);

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
}

pub mod divider;
pub mod text;

pub use divider::{Direction, Divider};
pub use text::Text;
