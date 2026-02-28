# ccui

An ID-driven TUI framework built on ratatui and tokio.

## Quick Start

```rust
use ccui::{Ui, Text, Style, Container};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Ui::run()?;

    // Add widgets
    doc.add_widget("title", Text::new("Hello"))?;

    // Add container with children
    let row = doc.add_container("row", Style::new().row())?;
    row.add_widget("left", Text::new("Left"))?;
    row.add_widget("right", Text::new("Right"))?;

    // Handle events (this blocks and prevents program from exiting)
    while let Some(event) = doc.event_receiver().recv().await {
        if let ccui::Event::Key(key) = event {
            if key.code == crossterm::event::KeyCode::Char('q') {
                break;
            }
        }
    }

    Ok(())
}
```

## How It Works

### `Ui::run()` Behavior

`Ui::run()` does the following:
1. Initializes the terminal (alternate screen, raw mode)
2. Spawns an async render loop in a background tokio task
3. Returns a `Document` handle immediately

**Important**: The render loop runs in the background. If your main function returns, the program exits and the terminal is restored.

### Why Blocking is Required

```rust
let mut doc = Ui::run()?;  // Returns immediately

/* You can do other things here, like data processing or updating UI via APIs */

// ❌ Without blocking, program exits here
// Terminal is restored, render loop task is dropped

// ✅ With blocking event loop
while let Some(event) = doc.event_receiver().recv().await {
    // Program stays alive, render loop continues
}
```

### Architecture

```
┌─────────────────────────────────────────┐
│  Main Thread                            │
│  let doc = Ui::run()?;                  │
│  while event { ... }  ← blocks here     │
└─────────────────────────────────────────┘
              │
              │ ui_tx (commands)
              ↓
┌─────────────────────────────────────────┐
│  Render Loop (background tokio task)    │
│  - Receives commands via channel        │
│  - Renders UI at ~60 FPS                │
│  - Sends events via channel             │
└─────────────────────────────────────────┘
```

### Design Philosophy

**ID-driven + Async**

- All UI elements are identified by string IDs
- Operations are synchronous sends to an async render loop
- Handles (`ContainerHandle`, `WidgetHandle`) are lightweight and cloneable
- Event-driven: block on `event_receiver()` to stay alive and handle input

## Core Concepts

- **ID-driven**: All elements identified by string IDs
- **Handles**: `ContainerHandle` and `WidgetHandle` for chaining operations
- **Async**: All operations are synchronous sends to async render loop

## Notes

- **Blocking required**: `Ui::run()` returns immediately. You must block (e.g., event loop) to prevent program from exiting
- Default parent for `Document` methods is `"root"`
- `ContainerHandle` and `WidgetHandle` are cloneable
- Use `get_container(id)` / `get_widget(id)` to get handles by ID
- When `Document` is dropped, the terminal is automatically restored
