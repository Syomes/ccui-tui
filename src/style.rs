use ratatui::layout::Rect;

/// Flex direction - how children are arranged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    /// Horizontal arrangement (left to right).
    Row,
    /// Vertical arrangement (top to bottom).
    #[default]
    Column,
}

/// Spacing offset (padding).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RectOffset {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl RectOffset {
    pub fn all(value: u16) -> Self {
        RectOffset {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        RectOffset {
            top,
            right,
            bottom,
            left,
        }
    }
}

/// Style properties - shared by all nodes.
///
/// - Container nodes use `flex_direction`, `gap`, and `padding` for layout.
/// - Widget nodes may use `padding` to inset their rendering area.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Style {
    pub flex_direction: FlexDirection,
    pub gap: u16,
    pub padding: RectOffset,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn row(mut self) -> Self {
        self.flex_direction = FlexDirection::Row;
        self
    }

    pub fn column(mut self) -> Self {
        self.flex_direction = FlexDirection::Column;
        self
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn padding(mut self, padding: RectOffset) -> Self {
        self.padding = padding;
        self
    }

    pub fn padding_all(mut self, value: u16) -> Self {
        self.padding = RectOffset::all(value);
        self
    }

    /// Shrink the area by padding.
    pub fn shrink(&self, area: Rect) -> Rect {
        Rect::new(
            area.x + self.padding.left,
            area.y + self.padding.top,
            area.width
                .saturating_sub(self.padding.left + self.padding.right),
            area.height
                .saturating_sub(self.padding.top + self.padding.bottom),
        )
    }

    /// Calculate child areas based on flex direction.
    pub fn calculate_children_areas(&self, parent_area: Rect, n_children: usize) -> Vec<Rect> {
        if n_children == 0 {
            return vec![];
        }

        // Apply padding to get content area
        let content_x = parent_area.x + self.padding.left;
        let content_y = parent_area.y + self.padding.top;
        let content_w = parent_area
            .width
            .saturating_sub(self.padding.left + self.padding.right);
        let content_h = parent_area
            .height
            .saturating_sub(self.padding.top + self.padding.bottom);

        let total_gap = self.gap * (n_children as u16 - 1);

        match self.flex_direction {
            FlexDirection::Row => {
                let available_width = content_w.saturating_sub(total_gap);
                let child_width = available_width / n_children as u16;

                (0..n_children)
                    .map(|i| {
                        Rect::new(
                            content_x + (i as u16 * (child_width + self.gap)),
                            content_y,
                            child_width,
                            content_h,
                        )
                    })
                    .collect()
            }
            FlexDirection::Column => {
                let available_height = content_h.saturating_sub(total_gap);
                let child_height = available_height / n_children as u16;

                (0..n_children)
                    .map(|i| {
                        Rect::new(
                            content_x,
                            content_y + (i as u16 * (child_height + self.gap)),
                            content_w,
                            child_height,
                        )
                    })
                    .collect()
            }
        }
    }
}
