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

/// A container that can hold widgets and other containers.
pub trait Container {
    fn add_container(
        &self,
        id: impl Into<String>,
        style: Style,
    ) -> Result<ContainerHandle, mpsc::error::TrySendError<UiMessage>>;

    fn add_widget<C: Widget + 'static>(
        &self,
        id: impl Into<String>,
        widget: C,
    ) -> Result<WidgetHandle, mpsc::error::TrySendError<UiMessage>>;
}

/// Operations available on a widget handle.
pub trait WidgetOps {
    fn update<C: Widget + 'static>(
        &self,
        widget: C,
    ) -> Result<(), mpsc::error::TrySendError<UiMessage>>;

    fn remove(self) -> Result<(), mpsc::error::TrySendError<UiMessage>>;
}

/// Handle to the UI system.
///
/// `Document` serves as the event bus for the UI, providing:
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

impl Container for Document {
    fn add_container(
        &self,
        id: impl Into<String>,
        style: Style,
    ) -> Result<ContainerHandle, mpsc::error::TrySendError<UiMessage>> {
        let id = id.into();
        self.ui_tx.try_send(UiMessage::AddContainer {
            parent_id: "root".to_string(),
            id: id.clone(),
            style,
        })?;
        Ok(ContainerHandle {
            ui_tx: self.ui_tx.clone(),
            id,
        })
    }

    fn add_widget<C: Widget + 'static>(
        &self,
        id: impl Into<String>,
        widget: C,
    ) -> Result<WidgetHandle, mpsc::error::TrySendError<UiMessage>> {
        let id = id.into();
        let style = widget.node_style_hint().unwrap_or_default();
        self.ui_tx.try_send(UiMessage::AddWidget {
            parent_id: "root".to_string(),
            id: id.clone(),
            widget: Box::new(widget),
            style,
        })?;
        Ok(WidgetHandle {
            ui_tx: self.ui_tx.clone(),
            id,
        })
    }
}

impl Document {
    pub fn get_container(&self, id: impl Into<String>) -> ContainerHandle {
        ContainerHandle {
            ui_tx: self.ui_tx.clone(),
            id: id.into(),
        }
    }

    pub fn get_widget(&self, id: impl Into<String>) -> WidgetHandle {
        WidgetHandle {
            ui_tx: self.ui_tx.clone(),
            id: id.into(),
        }
    }

    pub fn update_widget<C: Widget + 'static>(
        &self,
        id: impl Into<String>,
        widget: C,
    ) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.ui_tx.try_send(UiMessage::UpdateWidget {
            id: id.into(),
            widget: Box::new(widget),
        })?;
        Ok(())
    }

    pub fn remove_widget(
        &self,
        id: impl Into<String>,
    ) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.ui_tx.try_send(UiMessage::RemoveWidget(id.into()))?;
        Ok(())
    }

    pub fn event_receiver(&mut self) -> &mut mpsc::Receiver<Event> {
        &mut self.event_rx
    }
}

/// Handle to a container.
#[derive(Clone)]
pub struct ContainerHandle {
    ui_tx: mpsc::Sender<UiMessage>,
    id: String,
}

impl ContainerHandle {
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl Container for ContainerHandle {
    fn add_container(
        &self,
        id: impl Into<String>,
        style: Style,
    ) -> Result<ContainerHandle, mpsc::error::TrySendError<UiMessage>> {
        let id = id.into();
        self.ui_tx.try_send(UiMessage::AddContainer {
            parent_id: self.id.clone(),
            id: id.clone(),
            style,
        })?;
        Ok(ContainerHandle {
            ui_tx: self.ui_tx.clone(),
            id,
        })
    }

    fn add_widget<C: Widget + 'static>(
        &self,
        id: impl Into<String>,
        widget: C,
    ) -> Result<WidgetHandle, mpsc::error::TrySendError<UiMessage>> {
        let id = id.into();
        let style = widget.node_style_hint().unwrap_or_default();
        self.ui_tx.try_send(UiMessage::AddWidget {
            parent_id: self.id.clone(),
            id: id.clone(),
            widget: Box::new(widget),
            style,
        })?;
        Ok(WidgetHandle {
            ui_tx: self.ui_tx.clone(),
            id,
        })
    }
}

/// Handle to a widget.
#[derive(Clone)]
pub struct WidgetHandle {
    ui_tx: mpsc::Sender<UiMessage>,
    id: String,
}

impl WidgetHandle {
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl WidgetOps for WidgetHandle {
    fn update<C: Widget + 'static>(
        &self,
        widget: C,
    ) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.ui_tx.try_send(UiMessage::UpdateWidget {
            id: self.id.clone(),
            widget: Box::new(widget),
        })?;
        Ok(())
    }

    fn remove(self) -> Result<(), mpsc::error::TrySendError<UiMessage>> {
        self.ui_tx.try_send(UiMessage::RemoveWidget(self.id))?;
        Ok(())
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
