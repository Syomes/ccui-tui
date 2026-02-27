use ratatui::{Frame, layout::Rect};
use crate::component::Component;
use crate::style::Style;

/// A horizontal layout container that arranges child components in a row.
///
/// Note: Child components should be added as separate nodes in the tree.
/// The node containing this component will automatically use horizontal layout direction.
pub struct Row;

impl Row {
    pub fn new() -> Self {
        Row
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Row {
    fn render(&self, _f: &mut Frame, _area: Rect) {
        // Row is a container - children are rendered by the Node tree
    }
    
    fn node_style_hint(&self) -> Option<Style> {
        Some(Style::new().row())
    }
}
