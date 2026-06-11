use crossterm::event::{KeyCode, KeyEvent};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for an event listener.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ListenerId(u64);

impl ListenerId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        ListenerId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Trait for widget-specific messages.
/// Each widget defines its own message types and implements this trait.
pub trait WidgetMessage: Send + 'static {
    /// Apply this message to a widget.
    /// Called in the RenderLoop thread.
    fn apply(self: Box<Self>, widget: &mut dyn crate::widget::Widget);
}

/// Event types that can be listened to.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EventType {
    Click,
    DoubleClick,
    RightClick,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    Hover,
    Focus,
    Blur,
    KeyPress(KeyCode),
}

/// Context passed to event listeners.
#[derive(Clone)]
pub struct EventContext {
    pub event_type: EventType,
    /// Original event target (the node that initially triggered the event)
    pub target_id: String,
    /// Current target (the node whose listener is currently being invoked)
    pub current_target_id: String,
    pub mouse_x: Option<u16>,
    pub mouse_y: Option<u16>,
    pub scroll_delta: Option<i32>,
    pub key_code: Option<KeyCode>,
    /// Whether event propagation should stop
    pub propagation_stopped: bool,
}

impl EventContext {
    /// Create a new EventContext with sensible defaults.
    /// `target_id` is used for both `target_id` and `current_target_id`.
    pub fn new(event_type: EventType, target_id: &str) -> Self {
        let id = target_id.to_string();
        EventContext {
            event_type,
            current_target_id: id.clone(),
            target_id: id,
            mouse_x: None,
            mouse_y: None,
            scroll_delta: None,
            key_code: None,
            propagation_stopped: false,
        }
    }

    /// Set mouse coordinates on this context.
    pub fn with_mouse(mut self, x: u16, y: u16) -> Self {
        self.mouse_x = Some(x);
        self.mouse_y = Some(y);
        self
    }

    /// Set key code on this context.
    pub fn with_key(mut self, code: KeyCode) -> Self {
        self.key_code = Some(code);
        self
    }

    /// Set scroll delta on this context.
    pub fn with_scroll(mut self, delta: i32) -> Self {
        self.scroll_delta = Some(delta);
        self
    }

    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }
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
    UpdateStyle {
        id: String,
        style: crate::style::Style,
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
    AddGlobalListener {
        event_type: EventType,
        listener: EventListener,
        listener_id: ListenerId,
    },

    // Widget-specific messages
    WidgetMessage {
        id: String,
        message: Box<dyn WidgetMessage>,
    },

    // Mouse capture toggle
    ToggleMouseCapture,
}

/// Events received from the terminal (keyboard, mouse, resize).
#[derive(Clone, Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
}
