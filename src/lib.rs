//! ccui - A TUI framework built on ratatui and tokio.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use ccui::{Ui, Text, Column, Row, Event};
//! use crossterm::event::KeyCode;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut doc = Ui::run()?;
//!     
//!     // Add a simple text
//!     doc.add_component("root".into(), "title".into(), Text::new("Hello, World!")).await?;
//!     
//!     // Add a horizontal layout with children
//!     doc.add_component("root".into(), "row".into(), Row::new()).await?;
//!     doc.add_component("row".into(), "left".into(), Text::new("Left")).await?;
//!     doc.add_component("row".into(), "right".into(), Text::new("Right")).await?;
//!     
//!     // Add a vertical layout
//!     doc.add_component("root".into(), "col".into(), Column::new()).await?;
//!     doc.add_component("col".into(), "item1".into(), Text::new("Item 1")).await?;
//!     doc.add_component("col".into(), "item2".into(), Text::new("Item 2")).await?;
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
mod component;
mod document;
mod event;
mod internal;
mod style;

// Re-export public API
pub use component::{Component, Column, Row, Text};
pub use document::{Document, Ui};
pub use event::{Event, UiMessage};
pub use style::{Style, Display, FlexDirection, Dimension, RectOffset};

// Internal modules are not re-exported
