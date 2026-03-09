use ratatui::layout::Rect;

use crate::internal::Node;
use crate::style::{FlexDirection, Style};

/// Layout context - encapsulates direction-related coordinate calculations.
pub struct LayoutContext {
    pub content: Rect,
    pub direction: FlexDirection,
}

impl LayoutContext {
    /// Main axis size (Row=width, Column=height).
    pub fn main_size(&self) -> u16 {
        match self.direction {
            FlexDirection::Row => self.content.width,
            FlexDirection::Column => self.content.height,
        }
    }

    /// Cross axis size (Row=height, Column=width).
    pub fn cross_size(&self) -> u16 {
        match self.direction {
            FlexDirection::Row => self.content.height,
            FlexDirection::Column => self.content.width,
        }
    }

    /// Build Rect from main axis position and size.
    pub fn build_area(&self, main_pos: u16, main_size: u16) -> Rect {
        match self.direction {
            FlexDirection::Row => Rect::new(
                self.content.x + main_pos,
                self.content.y,
                main_size,
                self.content.height,
            ),
            FlexDirection::Column => Rect::new(
                self.content.x,
                self.content.y + main_pos,
                self.content.width,
                main_size,
            ),
        }
    }

    /// Get child's expected size on main axis.
    pub fn get_child_size(&self, child: &Node) -> u16 {
        child
            .widget
            .as_ref()
            .and_then(|w| w.size_hint())
            .map_or(0, |(w, h)| match self.direction {
                FlexDirection::Row => w,
                FlexDirection::Column => h,
            })
    }
}

/// Calculate content area (applying border and padding).
pub fn content_area(style: &Style, parent_area: Rect) -> Rect {
    let after_border = shrink_border(style, parent_area);
    Rect::new(
        after_border.x + style.padding.left,
        after_border.y + style.padding.top,
        after_border
            .width
            .saturating_sub(style.padding.left + style.padding.right),
        after_border
            .height
            .saturating_sub(style.padding.top + style.padding.bottom),
    )
}

/// Shrink area by border.
pub fn shrink_border(style: &Style, area: Rect) -> Rect {
    if style.border_type.is_none() {
        return area;
    }
    Rect::new(
        area.x + 1,
        area.y + 1,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    )
}

/// Check if node is floating.
#[inline]
fn is_floating(child: &Node) -> bool {
    child.style.position_mode == crate::style::PositionMode::Floating
}

/// Calculate child layout areas (for Node::layout).
pub fn calculate_children_areas(style: &Style, parent_area: Rect, children: &[Node]) -> Vec<Rect> {
    if children.is_empty() {
        return vec![];
    }

    let content = content_area(style, parent_area);
    let ctx = LayoutContext {
        content,
        direction: style.flex_direction,
    };

    // Check if all children are floating
    if children.iter().all(is_floating) {
        return vec![parent_area; children.len()];
    }

    // Calculate sizes
    let sizes = calculate_child_sizes(style, children, &ctx);

    // Build areas
    let mut main_pos = 0u16;
    children
        .iter()
        .zip(sizes)
        .map(|(child, size)| {
            if is_floating(child) {
                return parent_area;
            }
            let area = ctx.build_area(main_pos, size);
            main_pos += size + style.gap;
            area
        })
        .collect()
}

/// Calculate child sizes on main axis.
fn calculate_child_sizes(style: &Style, children: &[Node], ctx: &LayoutContext) -> Vec<u16> {
    match style.layout_mode {
        crate::style::LayoutMode::Tiled => calc_tiled_sizes(style, children, ctx),
        crate::style::LayoutMode::Auto => calc_auto_sizes(style, children, ctx),
    }
}

/// Tiled mode: fixed size + flexible equal distribution.
fn calc_tiled_sizes(_style: &Style, children: &[Node], ctx: &LayoutContext) -> Vec<u16> {
    // Calculate fixed size sum + flexible node count
    let (fixed_sum, flexible_count) =
        children
            .iter()
            .filter(|c| !is_floating(c))
            .fold((0u16, 0u16), |(sum, count), child| {
                let size = ctx.get_child_size(child);
                if size > 0 {
                    (sum + size, count)
                } else {
                    (sum, count + 1)
                }
            });

    // Calculate flexible size
    let flexible_size = ctx
        .main_size()
        .saturating_sub(fixed_sum)
        .div_ceil(flexible_count.max(1));

    // Build size vector
    children
        .iter()
        .map(|child| {
            if is_floating(child) {
                0
            } else {
                let size = ctx.get_child_size(child);
                if size > 0 { size } else { flexible_size }
            }
        })
        .collect()
}

/// Auto mode: size based on content.
fn calc_auto_sizes(_style: &Style, children: &[Node], ctx: &LayoutContext) -> Vec<u16> {
    let available = Rect::new(0, 0, ctx.main_size(), ctx.cross_size());

    children
        .iter()
        .map(|child| {
            if is_floating(child) {
                0
            } else {
                let (w, h) = child.calculate_content_size(available);
                match ctx.direction {
                    FlexDirection::Row => w,
                    FlexDirection::Column => h,
                }
            }
        })
        .collect()
}

/// Calculate node content size (for Auto layout and future min/max constraints).
pub fn calculate_content_size(node: &Node, available_area: Rect) -> (u16, u16) {
    if let Some(widget) = &node.widget {
        // Leaf: widget content + padding + border
        let (w, h) = widget.content_size(available_area);
        let border_w = if node.style.border_type.is_some() {
            2
        } else {
            0
        };
        let border_h = if node.style.border_type.is_some() {
            2
        } else {
            0
        };

        (
            w + node.style.padding.left + node.style.padding.right + border_w,
            h + node.style.padding.top + node.style.padding.bottom + border_h,
        )
    } else if !node.children.is_empty() {
        // Container: recursively calculate children
        let ctx = LayoutContext {
            content: available_area,
            direction: node.style.flex_direction,
        };

        // Calculate children total size (returns (width, height))
        let (children_w, children_h) = calculate_container_content(node, &ctx);

        // Add padding and border
        let border_w = if node.style.border_type.is_some() {
            2
        } else {
            0
        };
        let border_h = if node.style.border_type.is_some() {
            2
        } else {
            0
        };

        (
            children_w + node.style.padding.left + node.style.padding.right + border_w,
            children_h + node.style.padding.top + node.style.padding.bottom + border_h,
        )
    } else {
        // Empty container
        (0, 0)
    }
}

/// Calculate container content size (excluding own padding/border).
/// Returns (total_width, total_height).
fn calculate_container_content(node: &Node, ctx: &LayoutContext) -> (u16, u16) {
    // Available space (minus own padding/border)
    let inner_available = Rect::new(0, 0, ctx.main_size(), ctx.cross_size());

    match ctx.direction {
        FlexDirection::Row => {
            // Row: children laid out horizontally
            let mut total_width = 0u16;
            let mut max_height = 0u16;

            for child in &node.children {
                let (w, h) = calculate_content_size(child, inner_available);
                total_width += w + node.style.gap;
                max_height = max_height.max(h);
            }

            (total_width.saturating_sub(node.style.gap), max_height)
        }
        FlexDirection::Column => {
            // Column: children laid out vertically
            let mut max_width = 0u16;
            let mut total_height = 0u16;

            for child in &node.children {
                let (w, h) = calculate_content_size(child, inner_available);
                max_width = max_width.max(w);
                total_height += h + node.style.gap;
            }

            (max_width, total_height.saturating_sub(node.style.gap))
        }
    }
}
