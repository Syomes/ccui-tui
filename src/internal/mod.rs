mod render;

pub use render::RenderLoop;

use ratatui::{Frame, layout::Rect};
use crate::component::Component;
use crate::style::Style;

/// Internal node in the UI tree.
///
/// Nodes form a hierarchical structure similar to DOM. Each node can optionally
/// have a component that renders content within its allocated area.
pub struct Node {
    pub id: String,
    pub style: Style,
    pub area: Rect,
    pub component: Option<Box<dyn Component>>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(id: String) -> Self {
        Node {
            id,
            style: Style::new().column(),  // root default: vertical layout
            area: Rect::default(),
            component: None,
            children: vec![],
        }
    }

    /// Layout the tree starting from this node.
    pub fn layout(&mut self, parent_area: Rect) {
        // Calculate this node's area
        self.area = self.style.calculate_area(parent_area);
        
        // Layout children
        let child_areas = self.style.calculate_children_areas(self.area, self.children.len());
        for (child, area) in self.children.iter_mut().zip(child_areas) {
            child.layout(area);
        }
    }

    pub fn render(&self, f: &mut Frame) {
        // First render this node's component if it has one
        if let Some(component) = &self.component {
            component.render(f, self.area);
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

    pub fn add_child_box(
        &mut self,
        parent_id: &str,
        id: String,
        component: Box<dyn Component>,
    ) {
        if let Some(parent) = self.find_child_mut(parent_id) {
            // Get style hint from component for the new node
            let style = component.node_style_hint().unwrap_or_default();
            
            parent.children.push(Node {
                id,
                style,
                area: Rect::default(),
                component: Some(component),
                children: vec![],
            });
        }
    }

    pub fn remove_child(&mut self, id: &str) {
        if self.id == id {
            self.component = None;
            self.children.clear();
            return;
        }
        self.children.retain(|child| child.id != id);
    }

    pub fn update_child_box(&mut self, id: &str, component: Box<dyn Component>) {
        if let Some(node) = self.find_child_mut(id) {
            node.component = Some(component);
        }
    }
}
