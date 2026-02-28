//! ccui - An ID-driven TUI framework built on ratatui and tokio.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use ccui::{Ui, Text, Style, Container};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut doc = Ui::run()?;
//!
//!     // Add widgets to root
//!     doc.add_widget("title", Text::new("Hello, World!"))?;
//!
//!     // Add a container and children
//!     let row = doc.add_container("row", Style::new().row())?;
//!     row.add_widget("left", Text::new("Left"))?;
//!     row.add_widget("right", Text::new("Right"))?;
//!
//!     // Handle events
//!     while let Some(event) = doc.event_receiver().recv().await {
//!         match event {
//!             Event::Key(key) => {
//!                 match key.code {
//!                     KeyCode::Char('q') => break,  // Quit on 'q'
//!                     KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => break,
//!                     _ => {}
//!                 }
//!             }
//!             Event::Mouse(mouse) => {
//!                 // Handle mouse clicks
//!             }
//!             Event::Resize(w, h) => {
//!                 // Handle terminal resize
//!             }
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

// Core modules
mod document;
mod event;
mod internal;
mod style;
mod widget;

// Re-export public API
pub use document::{Container, ContainerHandle, Document, Ui, WidgetHandle, WidgetOps};
pub use event::Event;
pub use style::{FlexDirection, RectOffset, Style};
pub use widget::{Text, Widget};
