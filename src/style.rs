use ratatui::layout::Rect;

/// Layout mode - how the node occupies space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Display {
    /// Tiled layout (default) - automatically fills parent space.
    #[default]
    Tiled,
    
    /// Floating layout - fixed position, can overlap.
    Floating { x: u16, y: u16 },
}

/// Flex direction - how children are arranged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    /// Horizontal arrangement (left to right).
    Row,
    /// Vertical arrangement (top to bottom).
    #[default]
    Column,
}

/// Dimension unit for width/height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dimension {
    /// Auto-size based on content or available space.
    #[default]
    Auto,
    /// Fixed size in terminal cells.
    Fixed(u16),
    /// Percentage of parent size (0-100).
    Percent(u16),
}

/// Spacing offset (padding or margin).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RectOffset {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl RectOffset {
    pub fn all(value: u16) -> Self {
        RectOffset { top: value, right: value, bottom: value, left: value }
    }
    
    pub fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        RectOffset { top, right, bottom, left }
    }
}

/// Style properties for layout and appearance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    pub display: Display,
    pub flex_direction: FlexDirection,
    pub width: Dimension,
    pub height: Dimension,
    pub gap: u16,
    pub padding: RectOffset,
    pub margin: RectOffset,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            display: Display::Tiled,
            flex_direction: FlexDirection::Column,
            width: Dimension::Auto,
            height: Dimension::Auto,
            gap: 0,
            padding: RectOffset::default(),
            margin: RectOffset::default(),
        }
    }
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }
    
    // === Display ===
    
    pub fn tiled(mut self) -> Self {
        self.display = Display::Tiled;
        self
    }
    
    pub fn floating(mut self, x: u16, y: u16) -> Self {
        self.display = Display::Floating { x, y };
        self
    }
    
    // === Flex Direction ===
    
    pub fn row(mut self) -> Self {
        self.flex_direction = FlexDirection::Row;
        self
    }
    
    pub fn column(mut self) -> Self {
        self.flex_direction = FlexDirection::Column;
        self
    }
    
    // === Dimensions ===
    
    pub fn width(mut self, width: Dimension) -> Self {
        self.width = width;
        self
    }
    
    pub fn height(mut self, height: Dimension) -> Self {
        self.height = height;
        self
    }
    
    pub fn fixed_size(mut self, width: u16, height: u16) -> Self {
        self.width = Dimension::Fixed(width);
        self.height = Dimension::Fixed(height);
        self
    }
    
    pub fn percent_size(mut self, width: u16, height: u16) -> Self {
        self.width = Dimension::Percent(width);
        self.height = Dimension::Percent(height);
        self
    }
    
    // === Spacing ===
    
    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }
    
    pub fn padding(mut self, padding: RectOffset) -> Self {
        self.padding = padding;
        self
    }
    
    pub fn margin(mut self, margin: RectOffset) -> Self {
        self.margin = margin;
        self
    }
    
    pub fn padding_all(mut self, value: u16) -> Self {
        self.padding = RectOffset::all(value);
        self
    }
    
    pub fn margin_all(mut self, value: u16) -> Self {
        self.margin = RectOffset::all(value);
        self
    }
    
    // === Layout Calculation ===
    
    /// Calculate the actual area for this node.
    pub fn calculate_area(&self, parent_area: Rect) -> Rect {
        match self.display {
            Display::Tiled => {
                // Apply margin
                let x = parent_area.x + self.margin.left as u16;
                let y = parent_area.y + self.margin.top as u16;
                let w = parent_area.width.saturating_sub(self.margin.left as u16 + self.margin.right as u16);
                let h = parent_area.height.saturating_sub(self.margin.top as u16 + self.margin.bottom as u16);
                
                // Apply dimension constraints
                let w = match self.width {
                    Dimension::Fixed(w) => w.min(w),
                    Dimension::Percent(p) => (w as u32 * p as u32 / 100) as u16,
                    Dimension::Auto => w,
                };
                let h = match self.height {
                    Dimension::Fixed(h) => h.min(h),
                    Dimension::Percent(p) => (h as u32 * p as u32 / 100) as u16,
                    Dimension::Auto => h,
                };
                
                Rect::new(x, y, w, h)
            }
            Display::Floating { x, y } => {
                let w = match self.width {
                    Dimension::Fixed(w) => w,
                    _ => parent_area.width / 2,
                };
                let h = match self.height {
                    Dimension::Fixed(h) => h,
                    _ => parent_area.height / 2,
                };
                Rect::new(x, y, w, h)
            }
        }
    }
    
    /// Calculate child areas based on flex direction.
    pub fn calculate_children_areas(&self, parent_area: Rect, n_children: usize) -> Vec<Rect> {
        if n_children == 0 {
            return vec![];
        }
        
        // Apply padding to get content area
        let content_x = parent_area.x + self.padding.left as u16;
        let content_y = parent_area.y + self.padding.top as u16;
        let content_w = parent_area.width.saturating_sub(self.padding.left as u16 + self.padding.right as u16);
        let content_h = parent_area.height.saturating_sub(self.padding.top as u16 + self.padding.bottom as u16);
        
        let total_gap = self.gap * (n_children as u16 - 1);
        
        match self.flex_direction {
            FlexDirection::Row => {
                let available_width = content_w.saturating_sub(total_gap);
                let child_width = available_width / n_children as u16;
                
                (0..n_children).map(|i| {
                    Rect::new(
                        content_x + (i as u16 * (child_width + self.gap)),
                        content_y,
                        child_width,
                        content_h,
                    )
                }).collect()
            }
            FlexDirection::Column => {
                let available_height = content_h.saturating_sub(total_gap);
                let child_height = available_height / n_children as u16;
                
                (0..n_children).map(|i| {
                    Rect::new(
                        content_x,
                        content_y + (i as u16 * (child_height + self.gap)),
                        content_w,
                        child_height,
                    )
                }).collect()
            }
        }
    }
}
