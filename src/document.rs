use crossterm::{
    ExecutableCommand,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

use crate::event::{Event, UiMessage};
use crate::internal::RenderLoop;
use crate::style::Style;
use crate::widget::Widget;

/// Handle to the UI system.
///
/// `Document` serves as the event bus for the UI, providing:
/// - Methods to send commands to the internal render loop (`add_widget`, `remove_widget`, `update_widget`)
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
    pub async fn add_widget<C: Widget + 'static>(
        &self,
        parent_id: String,
        id: String,
        widget: C,
    ) -> Result<(), mpsc::error::SendError<UiMessage>> {
        let style = widget.node_style_hint().unwrap_or_default();
        self.ui_tx
            .send(UiMessage::AddWidget {
                parent_id,
                id,
                widget: Box::new(widget),
                style,
            })
            .await
    }

    pub async fn add_widget_with_style<C: Widget + 'static>(
        &self,
        parent_id: String,
        id: String,
        widget: C,
        style: Style,
    ) -> Result<(), mpsc::error::SendError<UiMessage>> {
        self.ui_tx
            .send(UiMessage::AddWidget {
                parent_id,
                id,
                widget: Box::new(widget),
                style,
            })
            .await
    }

    pub async fn add_container(
        &self,
        parent_id: String,
        id: String,
        style: Style,
    ) -> Result<(), mpsc::error::SendError<UiMessage>> {
        self.ui_tx
            .send(UiMessage::AddContainer {
                parent_id,
                id,
                style,
            })
            .await
    }

    pub async fn remove_widget(&self, id: String) -> Result<(), mpsc::error::SendError<UiMessage>> {
        self.ui_tx.send(UiMessage::RemoveWidget(id)).await
    }

    pub async fn update_widget<C: Widget + 'static>(
        &self,
        id: String,
        widget: C,
    ) -> Result<(), mpsc::error::SendError<UiMessage>> {
        self.ui_tx
            .send(UiMessage::UpdateWidget {
                id,
                widget: Box::new(widget),
            })
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
