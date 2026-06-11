//! ccui - An ID-driven TUI framework built on ratatui and tokio.
//!
//! # Quick Start
#![doc = concat!("```rust,no_run\n", include_str!("../examples/lib-doc.rs"), "\n```")]
// Core modules
mod document;
pub mod event;
mod internal;
pub mod layout;
pub mod style;
pub mod util;
pub mod widget;

pub use document::{Container, ContainerHandle, Document, Ui, WidgetHandle};
