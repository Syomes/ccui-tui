use ratatui::layout::Rect;

use crate::internal::Node;
use crate::style::{FlexDirection, Style};

use super::context::LayoutContext;

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

/// Calculate child sizes on main axis.
pub(super) fn calculate_child_sizes(
    style: &Style,
    children: &[Node],
    ctx: &LayoutContext,
) -> Vec<u16> {
    match style.layout_mode {
        crate::style::LayoutMode::Tiled => calc_tiled_sizes(style, children, ctx),
        crate::style::LayoutMode::Auto => calc_auto_sizes(style, children, ctx),
    }
}

/// Tiled mode: fixed size + flexible equal distribution.
fn calc_tiled_sizes(_style: &Style, children: &[Node], ctx: &LayoutContext) -> Vec<u16> {
    // Calculate fixed size sum + flexible node count
    let (fixed_sum, flexible_count) = children
        .iter()
        .filter(|c| !super::area::is_floating(c))
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
            if super::area::is_floating(child) {
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
            if super::area::is_floating(child) {
                0
            } else {
                let (w, h) = calculate_content_size(child, available);
                match ctx.direction {
                    FlexDirection::Row => w,
                    FlexDirection::Column => h,
                }
            }
        })
        .collect()
}
