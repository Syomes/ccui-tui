use ratatui::layout::{Direction, Layout, Rect, Spacing};

/// Flex direction - how children are arranged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    /// Horizontal arrangement (left to right).
    Row,
    /// Vertical arrangement (top to bottom).
    #[default]
    Column,
}

/// Border type for containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderType {
    #[default]
    Plain, // ─ │ ┌ ┐ └ ┘
    Rounded, // ╭ ╮ ╰ ╯
    Double,  // ═ ║ ╔ ╗ ╚ ╝
    Thick,   // ━ ┃ ┏ ┓ ┗ ┛
}

/// Border configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Border {
    pub show: bool,
    pub border_type: BorderType,
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
    pub border: Border,
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

    pub fn border(mut self, border_type: BorderType) -> Self {
        self.border = Border {
            show: true,
            border_type,
        };
        self
    }

    pub fn no_border(mut self) -> Self {
        self.border = Border {
            show: false,
            border_type: BorderType::Plain,
        };
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

    /// Shrink the area by border.
    pub fn shrink_border(&self, area: Rect) -> Rect {
        if !self.border.show {
            return area;
        }
        // Shrink by 1 pixel for border, but allow overlap for border merging
        Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        )
    }

    /// Calculate child areas based on flex direction.
    pub fn calculate_children_areas(&self, parent_area: Rect, n_children: usize) -> Vec<Rect> {
        if n_children == 0 {
            return vec![];
        }

        // Apply border and padding to get content area
        let area_after_border = self.shrink_border(parent_area);

        let content_x = area_after_border.x + self.padding.left;
        let content_y = area_after_border.y + self.padding.top;
        let content_w = area_after_border
            .width
            .saturating_sub(self.padding.left + self.padding.right);
        let content_h = area_after_border
            .height
            .saturating_sub(self.padding.top + self.padding.bottom);

        let content_area = Rect::new(content_x, content_y, content_w, content_h);

        let constraints =
            vec![ratatui::layout::Constraint::Ratio(1, n_children as u32); n_children];

        let chunks = Layout::default()
            .direction(match self.flex_direction {
                FlexDirection::Row => Direction::Horizontal,
                FlexDirection::Column => Direction::Vertical,
            })
            .constraints(constraints)
            .spacing(Spacing::Overlap(self.gap))
            .split(content_area);

        chunks.to_vec()
    }
}
