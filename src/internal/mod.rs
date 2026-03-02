mod render;

pub use render::RenderLoop;

use crate::event::{EventContext, EventListener, EventType, ListenerId};
use crate::style::Style;
use crate::widget::Widget;
use ratatui::{Frame, layout::Rect};
use std::collections::HashMap;
use std::sync::Arc;

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
    pub area: Rect,         // Allocated area from layout
    pub content_area: Rect, // Actual content area (for hit testing)
    pub widget: Option<Box<dyn Widget>>,
    pub children: Vec<Node>,
    /// Event listeners attached to this node.
    pub listeners: HashMap<EventType, HashMap<ListenerId, EventListener>>,
}

impl Node {
    pub fn new(id: String) -> Self {
        Node {
            id,
            style: Style::new().column(),
            area: Rect::default(),
            content_area: Rect::default(),
            widget: None,
            children: vec![],
            listeners: HashMap::new(),
        }
    }

    /// Layout the tree starting from this node.
    pub fn layout(&mut self, parent_area: Rect) {
        // Calculate this node's area
        self.area = parent_area;

        // Calculate content area
        if let Some(widget) = &self.widget {
            let (w, h) = widget.content_size(self.area);
            self.content_area = Rect::new(self.area.x, self.area.y, w, h);
        } else {
            self.content_area = self.area;
        }

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

    /// Find the widget at the given position.
    /// Returns the id of the deepest child that contains the point.
    pub fn find_widget_at(&self, x: u16, y: u16) -> Option<String> {
        if !self.area.contains((x, y).into()) {
            return None;
        }

        for child in &self.children {
            if let Some(id) = child.find_widget_at(x, y) {
                return Some(id);
            }
        }

        // Check content area for widget hit testing
        if self.widget.is_some() && self.content_area.contains((x, y).into()) {
            Some(self.id.clone())
        } else if self.widget.is_none() {
            // Container without widget
            Some(self.id.clone())
        } else {
            None
        }
    }

    /// Trigger all listeners for the given event type.
    pub fn trigger_event(&self, event_type: &EventType, ctx: EventContext) {
        if let Some(listeners) = self.listeners.get(event_type) {
            for listener in listeners.values() {
                listener(ctx.clone());
            }
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
                content_area: Rect::default(),
                widget: Some(widget),
                children: vec![],
                listeners: HashMap::new(),
            });
        }
    }

    pub fn add_container(&mut self, parent_id: &str, id: String, style: Style) {
        if let Some(parent) = self.find_child_mut(parent_id) {
            parent.children.push(Node {
                id,
                style,
                area: Rect::default(),
                content_area: Rect::default(),
                widget: None,
                children: vec![],
                listeners: HashMap::new(),
            });
        }
    }

    pub fn remove_child(&mut self, id: &str) {
        if self.id == id {
            self.widget = None;
            self.children.clear();
            self.listeners.clear();
            return;
        }
        self.children.retain(|child| child.id != id);
    }

    pub fn update_widget_box(&mut self, id: &str, widget: Box<dyn Widget>) {
        if let Some(node) = self.find_child_mut(id) {
            node.widget = Some(widget);
        }
    }

    pub fn update_style(&mut self, id: &str, style: Style) {
        if let Some(node) = self.find_child_mut(id) {
            node.style = style;
        }
    }

    // Event system methods

    /// Add an event listener to a node.
    pub fn add_event_listener(
        &mut self,
        target_id: &str,
        event_type: EventType,
        listener: EventListener,
        listener_id: ListenerId,
    ) {
        if self.id == target_id {
            self.listeners
                .entry(event_type)
                .or_insert_with(HashMap::new)
                .insert(listener_id, listener);
        } else {
            // Recursively search in children
            for child in &mut self.children {
                child.add_event_listener(
                    target_id,
                    event_type.clone(),
                    Arc::clone(&listener),
                    listener_id,
                );
            }
        }
    }

    /// Remove an event listener by its ID.
    pub fn remove_event_listener(&mut self, listener_id: ListenerId) {
        for listeners in self.listeners.values_mut() {
            listeners.remove(&listener_id);
        }
        for child in &mut self.children {
            child.remove_event_listener(listener_id);
        }
    }
}
