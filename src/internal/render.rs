use crossterm::event::MouseEventKind;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

use crate::event::{Event, EventContext, EventType, UiMessage};
use crate::internal::Node;

/// Internal render loop state.
pub struct RenderLoop {
    root: Node,
}

impl RenderLoop {
    pub fn new() -> Self {
        RenderLoop {
            root: Node::new("root".to_string()),
        }
    }

    pub async fn run(
        mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
        mut ui_rx: mpsc::Receiver<UiMessage>,
        event_tx: mpsc::Sender<Event>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = Self::new();

        loop {
            // Handle UI commands
            while let Ok(msg) = ui_rx.try_recv() {
                match msg {
                    UiMessage::AddWidget {
                        parent_id,
                        id,
                        widget,
                        style,
                    } => {
                        state.root.add_widget_box(&parent_id, id, widget, style);
                    }
                    UiMessage::AddContainer {
                        parent_id,
                        id,
                        style,
                    } => {
                        state.root.add_container(&parent_id, id, style);
                    }
                    UiMessage::RemoveWidget(id) => {
                        state.root.remove_child(&id);
                    }
                    UiMessage::UpdateWidget { id, widget } => {
                        state.root.update_widget_box(&id, widget);
                    }
                    UiMessage::AddEventListener {
                        target_id,
                        event_type,
                        listener,
                        listener_id,
                    } => {
                        state.root.add_event_listener(
                            &target_id,
                            event_type,
                            listener,
                            listener_id,
                        );
                    }
                    UiMessage::RemoveEventListener { listener_id } => {
                        state.root.remove_event_listener(listener_id);
                    }
                }
            }

            // Poll terminal events and dispatch
            if let Ok(true) = crossterm::event::poll(std::time::Duration::ZERO) {
                if let Ok(event) = crossterm::event::read() {
                    match event {
                        crossterm::event::Event::Key(key) => {
                            // Forward key events to users via event_receiver
                            let _ = event_tx.send(Event::Key(key)).await;
                            // Future: dispatch to focused element
                        }
                        crossterm::event::Event::Mouse(mouse) => {
                            // Forward to users
                            let _ = event_tx.send(Event::Mouse(mouse.clone())).await;
                            // Dispatch to element under mouse
                            state.dispatch_mouse_event(mouse);
                        }
                        crossterm::event::Event::Resize(w, h) => {
                            let _ = event_tx.send(Event::Resize(w, h)).await;
                        }
                        _ => {}
                    }
                }
            }

            // Render the tree
            let _ = terminal.draw(|f| {
                // First calculate layout based on screen size
                let screen_area = f.area();
                state.root.layout(screen_area);

                // Then render
                state.root.render(f);
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
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
}
