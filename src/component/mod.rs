use ratatui::{Frame, layout::Rect};
use crate::style::Style;

/// A renderable component that can be displayed in a terminal area.
pub trait Component: Send + Sync {
    /// Render the component within the given frame and area.
    fn render(&self, f: &mut Frame, area: Rect);
    
    /// Get the style hint for the node that contains this component.
    fn node_style_hint(&self) -> Option<Style> {
        None
    }
}

pub mod column;
pub use column::Column;

pub mod row;
pub use row::Row;

pub mod text;
pub use text::Text;
