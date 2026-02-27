mod render;

pub use render::RenderLoop;

use crate::style::Style;
use crate::widget::Widget;
use ratatui::{Frame, layout::Rect};

/// Internal node in the UI tree.
///
/// Nodes form a hierarchical structure similar to DOM. Each node can optionally
/// have a widget that renders content within its allocated area.
///
/// Convention:
/// - Container node: `widget` is None, `children` is not empty
/// - Leaf node (widget): `widget` is Some, `children` is empty
pub struct Node {
    pub id: String,
    pub style: Style,
    pub area: Rect,
    pub widget: Option<Box<dyn Widget>>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(id: String) -> Self {
        Node {
            id,
            style: Style::new().column(), // root default: vertical layout
            area: Rect::default(),
            widget: None,
            children: vec![],
        }
    }

    /// Layout the tree starting from this node.
    pub fn layout(&mut self, parent_area: Rect) {
        // Calculate this node's area
        self.area = parent_area;

        // Layout children
        let child_areas = self
            .style
            .calculate_children_areas(self.area, self.children.len());
        for (child, area) in self.children.iter_mut().zip(child_areas) {
            child.layout(area);
        }
    }

    pub fn render(&self, f: &mut Frame) {
        // First render this node's widget if it has one
        if let Some(widget) = &self.widget {
            widget.render(f, self.area, &self.style);
        }

        // Then render all children
        for child in &self.children {
            child.render(f);
        }
    }

    fn find_child_mut(&mut self, id: &str) -> Option<&mut Node> {
        if self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_child_mut(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn add_widget_box(
        &mut self,
        parent_id: &str,
        id: String,
        widget: Box<dyn Widget>,
        style: Style,
    ) {
        if let Some(parent) = self.find_child_mut(parent_id) {
            parent.children.push(Node {
                id,
                style,
                area: Rect::default(),
                widget: Some(widget),
                children: vec![],
            });
        }
    }

    pub fn add_container(&mut self, parent_id: &str, id: String, style: Style) {
        if let Some(parent) = self.find_child_mut(parent_id) {
            parent.children.push(Node {
                id,
                style,
                area: Rect::default(),
                widget: None,
                children: vec![],
            });
        }
    }

    pub fn remove_child(&mut self, id: &str) {
        if self.id == id {
            self.widget = None;
            self.children.clear();
            return;
        }
        self.children.retain(|child| child.id != id);
    }

    pub fn update_widget_box(&mut self, id: &str, widget: Box<dyn Widget>) {
        if let Some(node) = self.find_child_mut(id) {
            node.widget = Some(widget);
        }
    }
}
