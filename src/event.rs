use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use crossterm::event::{KeyCode, KeyEvent};

/// Unique identifier for an event listener.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ListenerId(u64);

impl ListenerId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        ListenerId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Event types that can be listened to.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EventType {
    Click,
    DoubleClick,
    RightClick,
    ScrollUp,
    ScrollDown,
    Hover,
}

/// Context passed to event listeners.
#[derive(Clone)]
pub struct EventContext {
    pub event_type: EventType,
    pub target_id: String,
    pub mouse_x: Option<u16>,
    pub mouse_y: Option<u16>,
    pub scroll_delta: Option<i32>,
    pub key_code: Option<KeyCode>,
}

/// Event listener callback type.
pub type EventListener = Arc<dyn Fn(EventContext) + Send + Sync + 'static>;

/// Messages sent from external to the internal render loop.
pub enum UiMessage {
    AddWidget {
        parent_id: String,
        id: String,
        widget: Box<dyn crate::widget::Widget>,
        style: crate::style::Style,
    },
    AddContainer {
        parent_id: String,
        id: String,
        style: crate::style::Style,
    },
    RemoveWidget(String),
    UpdateWidget {
        id: String,
        widget: Box<dyn crate::widget::Widget>,
    },
    
    // Event system
    AddEventListener {
        target_id: String,
        event_type: EventType,
        listener: EventListener,
        listener_id: ListenerId,
    },
    RemoveEventListener {
        listener_id: ListenerId,
    },
}

/// Events received from the terminal (keyboard, mouse, resize).
#[derive(Clone, Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
}
