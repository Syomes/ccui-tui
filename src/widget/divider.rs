use crate::style::Style;
use crate::widget::Widget;
use ratatui::widgets::{Block, Borders};
use ratatui::{Frame, layout::Rect};

/// Direction of the divider line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Horizontal,
    Vertical,
}

/// Border type for dividers.
pub type LineType = crate::style::BorderType;

/// A divider widget that draws a line using Block borders.
pub struct Divider {
    direction: Direction,
    line_type: LineType,
}

impl Divider {
    /// Create a horizontal divider with plain line type.
    pub fn horizontal() -> Self {
        Divider {
            direction: Direction::Horizontal,
            line_type: LineType::Plain,
        }
    }

    /// Create a vertical divider with plain line type.
    pub fn vertical() -> Self {
        Divider {
            direction: Direction::Vertical,
            line_type: LineType::Plain,
        }
    }

    /// Set the line type.
    pub fn line_type(mut self, line_type: LineType) -> Self {
        self.line_type = line_type;
        self
    }
}

impl Widget for Divider {
    fn render(&self, f: &mut Frame, area: Rect, _style: &Style, _is_focused: bool) {
        // Convert our BorderType to ratatui's BorderType
        let border_type = match self.line_type {
            LineType::Plain => ratatui::widgets::BorderType::Plain,
            LineType::Rounded => ratatui::widgets::BorderType::Rounded,
            LineType::Double => ratatui::widgets::BorderType::Double,
            LineType::Thick => ratatui::widgets::BorderType::Thick,
        };

        let block = match self.direction {
            Direction::Horizontal => {
                // Horizontal divider: use BOTTOM border
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_type(border_type)
            }
            Direction::Vertical => {
                // Vertical divider: use RIGHT border
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_type(border_type)
            }
        };

        f.render_widget(block, area);
    }

    fn size_hint(&self) -> Option<(u16, u16)> {
        // Divider requests fixed size: 1 character in its direction
        Some(match self.direction {
            Direction::Horizontal => (0, 1), // Full width, 1 height (for the border)
            Direction::Vertical => (1, 0),   // 1 width (for the border), full height
        })
    }

    fn content_size(&self, area: Rect) -> (u16, u16) {
        // Divider occupies the full dimension in its direction
        // and 1 unit in the other dimension
        match self.direction {
            Direction::Horizontal => (area.width, 1),
            Direction::Vertical => (1, area.height),
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
