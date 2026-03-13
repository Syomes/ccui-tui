use ratatui::layout::Rect;

use crate::internal::Node;
use crate::style::FlexDirection;

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
