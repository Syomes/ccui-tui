use crate::event::{UiMessage, WidgetMessage};
use crate::style::{BorderType, Style};
use crate::widget::{Widget, WidgetKind, WidgetType};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Widget as RatatuiWidget},
};
use std::any::Any;
use tokio::sync::mpsc;

/// Messages for Divider widget.
pub enum DividerMessage {
    SetDirection(Direction),
    SetLineType(BorderType),
}

impl WidgetMessage for DividerMessage {
    fn apply(self: Box<Self>, widget: &mut dyn Widget) {
        if let Some(divider) = widget.as_any_mut().downcast_mut::<Divider>() {
            match *self {
                DividerMessage::SetDirection(dir) => divider.direction = dir,
                DividerMessage::SetLineType(line_type) => divider.line_type = line_type,
            }
        }
    }
}

/// Handle for controlling a Divider widget.
#[derive(Clone)]
pub struct DividerHandle {
    id: String,
    style: crate::style::Style,
    ui_tx: mpsc::Sender<UiMessage>,
}

impl crate::document::WidgetHandle for DividerHandle {
    fn id(&self) -> &str {
        &self.id
    }
    fn style(&self) -> &crate::style::Style {
        &self.style
    }
    fn ui_tx(&self) -> &mpsc::Sender<UiMessage> {
        &self.ui_tx
    }
}

impl DividerHandle {
    /// Set the direction of the divider.
    pub fn set_direction(
        &self,
        direction: Direction,
    ) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.ui_tx.try_send(UiMessage::WidgetMessage {
            id: self.id.clone(),
            message: Box::new(DividerMessage::SetDirection(direction)),
        })?;
        Ok(())
    }

    /// Set the line type of the divider.
    pub fn set_line_type(
        &self,
        line_type: BorderType,
    ) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.ui_tx.try_send(UiMessage::WidgetMessage {
            id: self.id.clone(),
            message: Box::new(DividerMessage::SetLineType(line_type)),
        })?;
        Ok(())
    }
}

impl WidgetType for Divider {
    type Handle = DividerHandle;

    fn kind() -> WidgetKind {
        WidgetKind::Divider
    }

    fn create_handle(
        id: String,
        ui_tx: mpsc::Sender<UiMessage>,
        style: crate::style::Style,
    ) -> Self::Handle {
        DividerHandle { id, style, ui_tx }
    }
}

/// Direction of the divider line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Horizontal,
    Vertical,
}

/// A divider widget that draws a line using Block borders.
pub struct Divider {
    direction: Direction,
    line_type: BorderType,
}

impl Divider {
    pub fn new() -> Self {
        Divider {
            direction: Direction::Horizontal,
            line_type: BorderType::Plain,
        }
    }

    /// Create a horizontal divider.
    pub fn horizontal() -> Self {
        Divider {
            direction: Direction::Horizontal,
            line_type: BorderType::Plain,
        }
    }

    /// Create a vertical divider.
    pub fn vertical() -> Self {
        Divider {
            direction: Direction::Vertical,
            line_type: BorderType::Plain,
        }
    }

    /// Set the direction.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Set the line type.
    pub fn line_type(mut self, line_type: BorderType) -> Self {
        self.line_type = line_type;
        self
    }

    /// Create a divider with border.
    pub fn bordered(mut self, border_type: BorderType) -> Self {
        self.line_type = border_type;
        self
    }
}

impl Default for Divider {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Divider {
    fn render(&self, buffer: &mut Buffer, area: Rect, _style: &Style, _is_focused: bool) {
        // Convert our BorderType to ratatui's BorderType
        let border_type = match self.line_type {
            BorderType::Plain => ratatui::widgets::BorderType::Plain,
            BorderType::Rounded => ratatui::widgets::BorderType::Rounded,
            BorderType::Double => ratatui::widgets::BorderType::Double,
            BorderType::Thick => ratatui::widgets::BorderType::Thick,
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

        block.render(area, buffer);
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

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
