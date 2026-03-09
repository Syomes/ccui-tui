use ratatui::layout::Rect;

/// Color for background and foreground.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    #[default]
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Rgb(u8, u8, u8),
}

impl From<Color> for ratatui::style::Color {
    fn from(ccui_color: Color) -> Self {
        match ccui_color {
            Color::Reset => ratatui::style::Color::Reset,
            Color::Black => ratatui::style::Color::Black,
            Color::Red => ratatui::style::Color::Red,
            Color::Green => ratatui::style::Color::Green,
            Color::Yellow => ratatui::style::Color::Yellow,
            Color::Blue => ratatui::style::Color::Blue,
            Color::Magenta => ratatui::style::Color::Magenta,
            Color::Cyan => ratatui::style::Color::Cyan,
            Color::White => ratatui::style::Color::White,
            Color::Rgb(r, g, b) => ratatui::style::Color::Rgb(r, g, b),
        }
    }
}

/// Position mode for containers - how the container positions itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PositionMode {
    /// Position is determined by the parent (default).
    #[default]
    Normal,
    /// Position is determined by x, y coordinates (floating window).
    Floating,
}

/// Layout mode for containers - how children are arranged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutMode {
    /// Children are tiled to fill the available space (default).
    #[default]
    Tiled,
    /// Children size themselves based on content.
    Auto,
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

/// Border type for containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderType {
    #[default]
    Plain, // ─ │ ┌ ┐ └ ┘
    Rounded, // ╭ ╮ ╰ ╯
    Double,  // ═ ║ ╔ ╗ ╚ ╝
    Thick,   // ━ ┃ ┏ ┓ ┗ ┛
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
    pub border_type: Option<BorderType>,
    pub layout_mode: LayoutMode,

    // Position mode (for floating windows)
    pub position_mode: PositionMode,
    pub x: u16,
    pub y: u16,
    pub width: u16,  // 0 = use parent width
    pub height: u16, // 0 = use parent height

    // Background color (None = transparent)
    pub bg_color: Option<Color>,
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
        self.border_type = Some(border_type);
        self
    }

    pub fn no_border(mut self) -> Self {
        self.border_type = None;
        self
    }

    pub fn tiled(mut self) -> Self {
        self.layout_mode = LayoutMode::Tiled;
        self
    }

    pub fn auto(mut self) -> Self {
        self.layout_mode = LayoutMode::Auto;
        self
    }

    /// Set position mode to Floating with default values.
    pub fn floating(mut self) -> Self {
        self.position_mode = PositionMode::Floating;
        self.x = 0;
        self.y = 0;
        self.width = 0; // 0 = use parent width
        self.height = 0; // 0 = use parent height
        self
    }

    /// Set floating position.
    pub fn position(mut self, x: u16, y: u16) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set floating size (0 = use parent size).
    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set background color.
    pub fn bg_color(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    /// Remove background color (transparent).
    pub fn no_bg(mut self) -> Self {
        self.bg_color = None;
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
}
