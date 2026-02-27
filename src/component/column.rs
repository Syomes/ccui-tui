use ratatui::{Frame, layout::Rect};
use crate::component::Component;
use crate::style::Style;

/// A vertical layout container that arranges child components in a column.
///
/// Note: Child components should be added as separate nodes in the tree.
/// The parent node will automatically use vertical layout direction.
pub struct Column;

impl Column {
    pub fn new() -> Self {
        Column
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Column {
    fn render(&self, _f: &mut Frame, _area: Rect) {
        // Column is a container, children are rendered by the Node tree
    }

    fn node_style_hint(&self) -> Option<Style> {
        Some(Style::new().column())
    }
}
