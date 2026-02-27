//! ccui - A TUI framework built on ratatui and tokio.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use ccui::{Ui, Text, Style, Event};
//! use crossterm::event::KeyCode;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut doc = Ui::run()?;
//!
//!     // Add a simple text
//!     doc.add_widget("root".into(), "title".into(), Text::new("Hello, World!")).await?;
//!
//!     // Add a horizontal layout with children
//!     doc.add_container("root".into(), "row".into(), Style::new().row()).await?;
//!     doc.add_widget("row".into(), "left".into(), Text::new("Left")).await?;
//!     doc.add_widget("row".into(), "right".into(), Text::new("Right")).await?;
//!
//!     // Add a vertical layout
//!     doc.add_container("root".into(), "col".into(), Style::new().column()).await?;
//!     doc.add_widget("col".into(), "item1".into(), Text::new("Item 1")).await?;
//!     doc.add_widget("col".into(), "item2".into(), Text::new("Item 2")).await?;
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
pub use document::{Document, Ui};
pub use event::{Event, UiMessage};
pub use style::{FlexDirection, RectOffset, Style};
pub use widget::{Text, Widget};
