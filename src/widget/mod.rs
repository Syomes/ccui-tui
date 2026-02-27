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
}

pub mod text;
pub use text::Text;
