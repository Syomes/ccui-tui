use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

use crate::event::{Event, UiMessage};
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
                    UiMessage::AddComponent {
                        parent_id,
                        id,
                        component,
                    } => {
                        state.root.add_child_box(&parent_id, id, component);
                    }
                    UiMessage::RemoveComponent(id) => {
                        state.root.remove_child(&id);
                    }
                    UiMessage::UpdateComponent { id, component } => {
                        state.root.update_child_box(&id, component);
                    }
                }
            }

            // Poll terminal events and forward to handle
            if let Ok(true) = crossterm::event::poll(std::time::Duration::ZERO) {
                if let Ok(event) = crossterm::event::read() {
                    match event {
                        crossterm::event::Event::Key(key) => {
                            let _ = event_tx.send(Event::Key(key)).await;
                        }
                        crossterm::event::Event::Mouse(mouse) => {
                            let _ = event_tx.send(Event::Mouse(mouse)).await;
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
}
