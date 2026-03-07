//! ccui - An ID-driven TUI framework built on ratatui and tokio.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use ccui::{Ui, Text, Style, Container, EventType};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut doc = Ui::run()?;
//!
//!     // Add widgets
//!     doc.add_widget("title", Text::new("Hello"))?;
//!
//!     // Add container with children
//!     let row = doc.add_container("row", Style::new().row())?;
//!     row.add_widget("btn", Text::new("Click me"))?;
//!
//!     // Add event listener
//!     doc.add_event_listener("btn", EventType::Click, |ctx| {
//!         println!("Clicked at ({:?}, {:?})", ctx.mouse_x, ctx.mouse_y);
//!     })?;
//!
//!     // Handle events
//!     while let Some(event) = doc.event_receiver().recv().await {
//!         if let ccui::Event::Key(key) = event {
//!             if key.code == crossterm::event::KeyCode::Char('q') {
//!                 break;
//!             }
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

// Core modules
mod document;
pub mod event;
mod internal;
pub mod style;
pub mod util;
pub mod widget;

pub use document::{Container, ContainerHandle, Document, Ui, WidgetHandle, WidgetOps};
