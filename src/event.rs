use crate::style::Style;
use crate::widget::Widget;

/// Messages sent from external to the internal render loop.
pub enum UiMessage {
    AddWidget {
        parent_id: String,
        id: String,
        widget: Box<dyn Widget>,
        style: Style,
    },
    AddContainer {
        parent_id: String,
        id: String,
        style: Style,
    },
    RemoveWidget(String),
    UpdateWidget {
        id: String,
        widget: Box<dyn Widget>,
    },
}

/// Events received from the terminal (keyboard, mouse, resize).
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
}
