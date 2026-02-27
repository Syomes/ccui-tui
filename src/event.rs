use crate::component::Component;

/// Messages sent from external to the internal render loop.
pub enum UiMessage {
    AddComponent {
        parent_id: String,
        id: String,
        component: Box<dyn Component>,
    },
    RemoveComponent(String),
    UpdateComponent {
        id: String,
        component: Box<dyn Component>,
    },
}

/// Events received from the terminal (keyboard, mouse, resize).
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
}
