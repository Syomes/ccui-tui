use ratatui::layout::Rect;

use crate::internal::Node;
use crate::style::{Overflow, Style};

use super::context::LayoutContext;

/// Check if node is floating.
#[inline]
pub(super) fn is_floating(child: &Node) -> bool {
    child.style.position_mode == crate::style::PositionMode::Floating
}

/// Calculate content area (applying border and padding).
fn content_area(style: &Style, parent_area: Rect) -> Rect {
    // !Visible containers' children area will be handled by ScrollView
    let real_content = if style.overflow == Overflow::Visible {
        shrink_border(style, parent_area)
    } else {
        parent_area
    };

    Rect::new(
        real_content.x + style.padding.left,
        real_content.y + style.padding.top,
        real_content
            .width
            .saturating_sub(style.padding.left + style.padding.right),
        real_content
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
    let sizes = super::sizing::calculate_child_sizes(style, children, &ctx);

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
