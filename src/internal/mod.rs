mod render;

use ratatui::layout::Size;
use ratatui::widgets::StatefulWidget;
pub use render::RenderLoop;
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

use crate::event::{EventContext, EventListener, EventType, ListenerId};
use crate::layout::shrink_border;
use crate::style::{Overflow, Style};
use crate::widget::Widget;
use ratatui::{buffer::Buffer, layout::Rect};
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
    pub scroll_state: Option<ScrollViewState>,
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
            scroll_state: None,
            widget: None,
            children: vec![],
            listeners: HashMap::new(),
        }
    }

    /// Layout the tree starting from this node.
    pub fn layout(&mut self, parent_area: Rect) {
        // Calculate this node's area based on position_mode
        self.area = if self.style.position_mode == crate::style::PositionMode::Floating {
            // Floating: use x, y, width, height from style
            Rect::new(
                self.style.x,
                self.style.y,
                if self.style.width == 0 {
                    parent_area.width
                } else {
                    self.style.width
                },
                if self.style.height == 0 {
                    parent_area.height
                } else {
                    self.style.height
                },
            )
        } else {
            // Normal: use parent_area
            parent_area
        };

        // Calculate content area
        if let Some(widget) = &self.widget {
            let (w, h) = widget.content_size(self.area);
            self.content_area = Rect::new(self.area.x, self.area.y, w, h);
        } else {
            self.content_area = self.area;
        }

        // Layout children
        let child_areas =
            crate::layout::calculate_children_areas(&self.style, self.area, &self.children);
        for (child, area) in self.children.iter_mut().zip(child_areas) {
            child.layout(area);
        }
    }

    pub fn render(&mut self, buffer: &mut Buffer, focused_id: Option<&str>) {
        // If this node has a background color, clear the area first to cover underlying content
        if self.style.bg_color.is_some() {
            use ratatui::widgets::Widget as RatatuiWidget;
            ratatui::widgets::Clear.render(self.area, buffer);
        }

        // Render background if bg_color is set
        if let Some(bg_color) = self.style.bg_color {
            use ratatui::widgets::Widget as RatatuiWidget;
            let block = ratatui::widgets::Block::default()
                .style(ratatui::style::Style::default().bg(bg_color.into()));
            block.render(self.area, buffer);
        }

        // Render border for CONTAINERS (nodes without widget or with children)
        let mut scroll_view: Option<ScrollView> = None;
        if self.widget.is_none() || !self.children.is_empty() {
            if let Some(border_type) = self.style.border_type {
                use crate::style::BorderType;
                use ratatui::symbols::merge::MergeStrategy;
                use ratatui::widgets::{
                    Block, BorderType as RatatuiBorderType, Widget as RatatuiWidget,
                };

                let ratatui_border_type = match border_type {
                    BorderType::Plain => RatatuiBorderType::Plain,
                    BorderType::Rounded => RatatuiBorderType::Rounded,
                    BorderType::Double => RatatuiBorderType::Double,
                    BorderType::Thick => RatatuiBorderType::Thick,
                };

                let block = Block::default()
                    .border_type(ratatui_border_type)
                    .borders(ratatui::widgets::Borders::ALL)
                    .merge_borders(MergeStrategy::Exact);

                block.render(self.area, buffer);
            }

            // Handle overflow with or without ScrollView
            if self.style.overflow == Overflow::Visible {
                // TODO: offset in scroll_state
            } else {
                if self.scroll_state.is_none() {
                    self.scroll_state = Some(ScrollViewState::new());
                }
                let size = Size::new(self.area.width, self.area.height);
                match self.style.overflow {
                    Overflow::Hidden => {
                        scroll_view = Some(
                            ScrollView::new(size).scrollbars_visibility(ScrollbarVisibility::Never),
                        );
                    }
                    Overflow::Auto => {
                        scroll_view = Some(
                            ScrollView::new(size)
                                .scrollbars_visibility(ScrollbarVisibility::Automatic),
                        );
                    }
                    Overflow::Scroll => {
                        scroll_view = Some(
                            ScrollView::new(size)
                                .scrollbars_visibility(ScrollbarVisibility::Always),
                        );
                    }
                    _ => {}
                }
            }
        }

        // Get the inner buffer for widgets
        let widget_buffer = match scroll_view.as_mut() {
            Some(sv) => sv.buf_mut(),
            None => buffer,
        };

        // Render widget if present
        if let Some(widget) = &self.widget {
            // Check if this node is focused
            let is_focused = focused_id.map_or(false, |fid| fid == self.id);
            widget.render(widget_buffer, self.area, &self.style, is_focused);
        }

        // Render children sorted by z-index (higher z-index renders on top)
        let mut children: Vec<_> = self.children.iter_mut().collect();
        children.sort_by_key(|child| child.style.z_index);
        for child in children {
            child.render(widget_buffer, focused_id);
        }

        if let Some(v) = scroll_view {
            let area = shrink_border(&self.style, self.area);
            v.render(area, buffer, &mut self.scroll_state.as_mut().unwrap());
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

        // Check if this node has a widget
        if self.widget.is_some() {
            Some(self.id.clone())
        } else if self.children.is_empty() {
            // Container without widget and children
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
                scroll_state: None,
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
                scroll_state: None,
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

    /// Calculate the content size of this node (including children).
    pub fn calculate_content_size(&self, available_area: Rect) -> (u16, u16) {
        crate::layout::calculate_content_size(self, available_area)
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
