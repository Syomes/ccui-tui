use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture, KeyEvent, MouseEventKind},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

use crate::event::{Event, EventContext, EventType, ListenerId, UiMessage};
use crate::internal::Node;
use std::collections::HashMap;

/// Internal render loop state.
pub struct RenderLoop {
    root: Node,
    focused_id: Option<String>,
    mouse_capture_enabled: bool,
    global_listeners: HashMap<EventType, Vec<(ListenerId, crate::event::EventListener)>>,
}

impl RenderLoop {
    pub fn new() -> Self {
        RenderLoop {
            root: Node::new("root".to_string()),
            focused_id: None,
            mouse_capture_enabled: true, // Default: enabled
            global_listeners: HashMap::new(),
        }
    }

    pub async fn run(
        mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
        mut ui_rx: mpsc::Receiver<UiMessage>,
        event_tx: mpsc::Sender<Event>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = Self::new();

        loop {
            // Render the tree
            let _ = terminal.draw(|f| {
                // First calculate layout based on screen size
                let screen_area = f.area();
                state.root.layout(screen_area);

                // Then render with focus state
                state.root.render(f, state.focused_id.as_deref());
            });

            // Handle UI commands
            while let Ok(msg) = ui_rx.try_recv() {
                state.handle_ui_msg(msg);
            }

            // Poll terminal events and dispatch
            if let Ok(true) = crossterm::event::poll(std::time::Duration::ZERO) {
                if let Ok(event) = crossterm::event::read() {
                    match event {
                        crossterm::event::Event::Key(key) => {
                            // global listeners (always triggered)
                            state.trigger_global_listeners(&EventType::KeyPress(key.code), key);

                            // focused widget (if any)
                            if let Some(ref focused_id) = state.focused_id {
                                if let Some(node) = state.root.find_child_mut(focused_id) {
                                    if let Some(widget) = &mut node.widget {
                                        widget.handle_key(key);
                                    }

                                    // Trigger key press event listeners on the node
                                    let ctx = EventContext {
                                        event_type: EventType::KeyPress(key.code),
                                        target_id: focused_id.clone(),
                                        mouse_x: None,
                                        mouse_y: None,
                                        scroll_delta: None,
                                        key_code: Some(key.code),
                                    };
                                    node.trigger_event(&EventType::KeyPress(key.code), ctx);
                                }
                            }

                            // Forward to user
                            let _ = event_tx.try_send(Event::Key(key));
                        }
                        crossterm::event::Event::Mouse(mouse) => {
                            // Forward to user
                            let _ = event_tx.try_send(Event::Mouse(mouse.clone()));

                            // Handle click for focus
                            if mouse.kind
                                == MouseEventKind::Down(crossterm::event::MouseButton::Left)
                            {
                                let clicked_id = state.root.find_widget_at(mouse.column, mouse.row);

                                // Update focus
                                if clicked_id.as_ref() != state.focused_id.as_ref() {
                                    // Blur old
                                    if let Some(old_id) = state.focused_id.take() {
                                        let ctx = EventContext {
                                            event_type: EventType::Blur,
                                            target_id: old_id.clone(),
                                            mouse_x: None,
                                            mouse_y: None,
                                            scroll_delta: None,
                                            key_code: None,
                                        };
                                        state.root.trigger_event(&EventType::Blur, ctx);
                                    }

                                    // Focus new (if clicked on a widget)
                                    if let Some(ref id) = clicked_id {
                                        state.focused_id = Some(id.clone());
                                        let ctx = EventContext {
                                            event_type: EventType::Focus,
                                            target_id: id.clone(),
                                            mouse_x: Some(mouse.column),
                                            mouse_y: Some(mouse.row),
                                            scroll_delta: None,
                                            key_code: None,
                                        };
                                        state.root.trigger_event(&EventType::Focus, ctx);
                                    }
                                }

                                // Trigger click listeners (if clicked on a widget)
                                if let Some(ref id) = clicked_id {
                                    let ctx = EventContext {
                                        event_type: EventType::Click,
                                        target_id: id.clone(),
                                        mouse_x: Some(mouse.column),
                                        mouse_y: Some(mouse.row),
                                        scroll_delta: None,
                                        key_code: None,
                                    };
                                    state.root.trigger_event(&EventType::Click, ctx);
                                }
                            }

                            // Dispatch to element under mouse
                            state.dispatch_mouse_event(mouse);
                        }
                        crossterm::event::Event::Resize(w, h) => {
                            // Forward to user
                            let _ = event_tx.try_send(Event::Resize(w, h));
                        }
                        _ => {}
                    }
                }
            }

            // TODO: add user-configurable FPS limit here
            tokio::time::sleep(tokio::time::Duration::from_millis(0)).await;
        }
    }

    /// Trigger global listeners for an event type.
    fn trigger_global_listeners(&self, event_type: &EventType, key: KeyEvent) {
        if let Some(listeners) = self.global_listeners.get(event_type) {
            for (_, listener) in listeners {
                let ctx = EventContext {
                    event_type: event_type.clone(),
                    target_id: String::from("global"),
                    mouse_x: None,
                    mouse_y: None,
                    scroll_delta: None,
                    key_code: Some(key.code),
                };
                listener(ctx);
            }
        }
    }

    /// Dispatch mouse events to the element under the cursor.
    fn dispatch_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) {
        // Convert to EventType
        let event_type = match mouse.kind {
            MouseEventKind::Down(_) => EventType::Click,
            MouseEventKind::Up(_) => return,
            MouseEventKind::Drag(_) => return,
            MouseEventKind::Moved => EventType::Hover,
            MouseEventKind::ScrollUp => EventType::ScrollUp,
            MouseEventKind::ScrollDown => EventType::ScrollDown,
            _ => return,
        };

        // Hit test to find target element
        let target_id = match self.root.find_widget_at(mouse.column, mouse.row) {
            Some(id) => id,
            None => return,
        };

        // Build event context
        let ctx = EventContext {
            event_type: event_type.clone(),
            target_id: target_id.clone(),
            mouse_x: Some(mouse.column),
            mouse_y: Some(mouse.row),
            scroll_delta: match mouse.kind {
                MouseEventKind::ScrollUp => Some(1),
                MouseEventKind::ScrollDown => Some(-1),
                _ => None,
            },
            key_code: None,
        };

        // Trigger listeners on the target node
        if let Some(node) = self.root.find_child_mut(&target_id) {
            node.trigger_event(&event_type, ctx);
        }
    }

    /// Handle a UI message from the framework.
    fn handle_ui_msg(&mut self, msg: UiMessage) {
        match msg {
            UiMessage::AddWidget {
                parent_id,
                id,
                widget,
                style,
            } => {
                self.root.add_widget_box(&parent_id, id, widget, style);
            }
            UiMessage::AddContainer {
                parent_id,
                id,
                style,
            } => {
                self.root.add_container(&parent_id, id, style);
            }
            UiMessage::RemoveWidget(id) => {
                self.root.remove_child(&id);
            }
            UiMessage::UpdateWidget { id, widget } => {
                self.root.update_widget_box(&id, widget);
            }
            UiMessage::UpdateStyle { id, style } => {
                self.root.update_style(&id, style);
            }
            UiMessage::AddEventListener {
                target_id,
                event_type,
                listener,
                listener_id,
            } => {
                self.root
                    .add_event_listener(&target_id, event_type, listener, listener_id);
            }
            UiMessage::RemoveEventListener { listener_id } => {
                self.root.remove_event_listener(listener_id);
            }
            UiMessage::AddGlobalListener {
                event_type,
                listener,
                listener_id,
            } => {
                self.global_listeners
                    .entry(event_type)
                    .or_insert_with(Vec::new)
                    .push((listener_id, listener));
            }
            UiMessage::ToggleMouseCapture => {
                self.mouse_capture_enabled = !self.mouse_capture_enabled;
                if self.mouse_capture_enabled {
                    let _ = std::io::stdout().execute(EnableMouseCapture);
                } else {
                    let _ = std::io::stdout().execute(DisableMouseCapture);
                }
            }
            UiMessage::WidgetMessage { id, message } => {
                // Widget-specific message: let the widget handle it
                if let Some(node) = self.root.find_child_mut(&id) {
                    if let Some(widget) = &mut node.widget {
                        message.apply(&mut **widget);
                    }
                }
            }
        }
    }
}
