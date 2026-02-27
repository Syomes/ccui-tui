use crossterm::{
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

use crate::component::Component;
use crate::event::{Event, UiMessage};
use crate::internal::RenderLoop;

/// Handle to the UI system.
///
/// `Document` serves as the event bus for the UI, providing:
/// - Methods to send commands to the internal render loop (`add_component`, `remove_component`, `update_component`)
/// - A receiver to get terminal events from the internal loop (`event_receiver`)
///
/// When `Document` is dropped, the terminal is automatically restored to its original state.
pub struct Document {
    ui_tx: mpsc::Sender<UiMessage>,
    event_rx: mpsc::Receiver<Event>,
}

impl Drop for Document {
    fn drop(&mut self) {
        // Cleanup terminal on exit
        let _ = terminal::disable_raw_mode();
        let _ = std::io::stdout().execute(LeaveAlternateScreen);
    }
}

impl Document {
    pub async fn add_component<C: Component + 'static>(
        &self,
        parent_id: String,
        id: String,
        component: C,
    ) -> Result<(), mpsc::error::SendError<UiMessage>> {
        self.ui_tx
            .send(UiMessage::AddComponent {
                parent_id,
                id,
                component: Box::new(component),
            })
            .await
    }

    pub async fn remove_component(
        &self,
        id: String,
    ) -> Result<(), mpsc::error::SendError<UiMessage>> {
        self.ui_tx
            .send(UiMessage::RemoveComponent(id))
            .await
    }

    pub async fn update_component<C: Component + 'static>(
        &self,
        id: String,
        component: C,
    ) -> Result<(), mpsc::error::SendError<UiMessage>> {
        self.ui_tx
            .send(UiMessage::UpdateComponent { id, component: Box::new(component) })
            .await
    }

    pub fn event_receiver(&mut self) -> &mut mpsc::Receiver<Event> {
        &mut self.event_rx
    }
}

/// Main entry point for the UI system.
///
/// Call `Ui::run()` to start the UI render loop. It returns a `Document` handle
/// that can be used to interact with the UI.
///
/// When the `Document` is dropped (goes out of scope), the terminal is automatically
/// restored to its original state.
pub struct Ui;

impl Ui {
    pub fn run() -> Result<Document, Box<dyn std::error::Error>> {
        // Enter alternate screen and raw mode
        terminal::enable_raw_mode()?;
        std::io::stdout().execute(EnterAlternateScreen)?;

        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

        let (ui_tx, ui_rx) = mpsc::channel(100);
        let (event_tx, event_rx) = mpsc::channel(100);

        tokio::spawn(async move {
            if let Err(e) = RenderLoop::run(terminal, ui_rx, event_tx).await {
                eprintln!("Render error: {}", e);
            }
        });

        Ok(Document { ui_tx, event_rx })
    }
}
