use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture, KeyEvent, KeyModifiers, MouseEventKind},
};
use ratatui::{Terminal, backend::Backend};
use tokio::sync::mpsc;

use crate::event::{Event, EventContext, EventType, ListenerId, UiMessage};
use crate::internal::Node;
use std::collections::HashMap;
use std::io;

/// Trait for abstracting crossterm event polling, enabling mock event sources in tests.
pub(crate) trait EventSource {
    fn poll(&mut self, timeout: std::time::Duration) -> io::Result<bool>;
    fn read(&mut self) -> io::Result<crossterm::event::Event>;
}

/// Production event source backed by crossterm.
pub(crate) struct CrosstermEventSource;

impl EventSource for CrosstermEventSource {
    fn poll(&mut self, timeout: std::time::Duration) -> io::Result<bool> {
        crossterm::event::poll(timeout)
    }
    fn read(&mut self) -> io::Result<crossterm::event::Event> {
        crossterm::event::read()
    }
}

/// Internal render loop state.
pub struct RenderLoop {
    root: Node,
    focused_id: Option<String>,
    mouse_capture_enabled: bool,
    global_listeners: HashMap<EventType, Vec<(ListenerId, crate::event::EventListener)>>,
}

impl RenderLoop {
    pub fn new() -> Self {
        RenderLoop {
            root: Node::new("root".to_string()),
            focused_id: None,
            mouse_capture_enabled: true, // Default: enabled
            global_listeners: HashMap::new(),
        }
    }

    pub async fn run(
        mut terminal: Terminal<impl Backend>,
        mut ui_rx: mpsc::Receiver<UiMessage>,
        mut event_source: impl EventSource,
        event_tx: mpsc::Sender<Event>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = Self::new();

        loop {
            // Render the tree
            let _ = terminal.draw(|f| {
                // First calculate layout based on screen size
                let screen_area = f.area();
                state.root.layout(screen_area);

                // Then render with focus state
                let buffer = f.buffer_mut();
                state.root.render(buffer, state.focused_id.as_deref());
            });

            // Handle UI commands
            while let Ok(msg) = ui_rx.try_recv() {
                state.handle_ui_msg(msg);
            }
            if ui_rx.is_closed() {
                break Ok(());
            }

            // Poll terminal events and dispatch
            if let Ok(true) = event_source.poll(std::time::Duration::ZERO) {
                if let Ok(event) = event_source.read() {
                    match event {
                        crossterm::event::Event::Key(key) => {
                            // focused widget (if any)
                            if let Some(ref focused_id) = state.focused_id {
                                if let Some(node) = state.root.find_child_mut(focused_id) {
                                    if let Some(widget) = &mut node.widget {
                                        widget.handle_key(key);
                                    }

                                    // Trigger key press event listeners on the node with bubbling
                                    let ctx = EventContext::new(
                                        EventType::KeyPress(key.code),
                                        focused_id,
                                    )
                                    .with_key(key.code);
                                    state.root.trigger_event_with_bubble(
                                        &EventType::KeyPress(key.code),
                                        ctx,
                                    );
                                }
                            }

                            // Global listeners (triggered after bubbling)
                            state.trigger_global_listeners(&EventType::KeyPress(key.code), key);

                            // Forward to user
                            let _ = event_tx.try_send(Event::Key(key));
                        }
                        crossterm::event::Event::Mouse(mouse) => {
                            // Forward to user
                            let _ = event_tx.try_send(Event::Mouse(mouse.clone()));

                            // Handle click for focus
                            if mouse.kind
                                == MouseEventKind::Down(crossterm::event::MouseButton::Left)
                            {
                                let clicked_id = state.root.find_widget_at(mouse.column, mouse.row);

                                // Update focus
                                if clicked_id.as_ref() != state.focused_id.as_ref() {
                                    // Blur old
                                    if let Some(old_id) = state.focused_id.take() {
                                        let ctx = EventContext::new(EventType::Blur, &old_id);
                                        state.root.trigger_event_with_bubble(&EventType::Blur, ctx);
                                    }

                                    // Focus new (if clicked on a widget)
                                    if let Some(ref id) = clicked_id {
                                        state.focused_id = Some(id.clone());
                                        let ctx = EventContext::new(EventType::Focus, id)
                                            .with_mouse(mouse.column, mouse.row);
                                        state
                                            .root
                                            .trigger_event_with_bubble(&EventType::Focus, ctx);
                                    }
                                }

                                // Trigger click listeners with bubbling (if clicked on a widget)
                                if let Some(ref id) = clicked_id {
                                    let ctx = EventContext::new(EventType::Click, id)
                                        .with_mouse(mouse.column, mouse.row);
                                    state.root.trigger_event_with_bubble(&EventType::Click, ctx);
                                }
                            }

                            // Dispatch to element under mouse
                            state.dispatch_mouse_event(mouse);
                        }
                        crossterm::event::Event::Resize(w, h) => {
                            // Forward to user
                            let _ = event_tx.try_send(Event::Resize(w, h));
                        }
                        _ => {}
                    }
                }
            }

            // TODO: add user-configurable FPS limit here
            tokio::time::sleep(tokio::time::Duration::from_millis(0)).await;
        }
    }

    /// Trigger global listeners for an event type.
    fn trigger_global_listeners(&self, event_type: &EventType, key: KeyEvent) {
        if let Some(listeners) = self.global_listeners.get(event_type) {
            for (_, listener) in listeners {
                let ctx = EventContext::new(event_type.clone(), "global").with_key(key.code);
                listener(ctx);
            }
        }
    }

    /// Dispatch mouse events to the element under the cursor.
    fn dispatch_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) {
        // Convert to EventType
        let event_type = match mouse.kind {
            MouseEventKind::Down(_) => EventType::Click,
            MouseEventKind::Up(_) => return,
            MouseEventKind::Drag(_) => return,
            MouseEventKind::Moved => EventType::Hover,
            MouseEventKind::ScrollUp => EventType::ScrollUp,
            MouseEventKind::ScrollDown => EventType::ScrollDown,
            MouseEventKind::ScrollLeft => EventType::ScrollLeft,
            MouseEventKind::ScrollRight => EventType::ScrollRight,
        };

        // For scroll events: find the scrollview container at the mouse position
        // For other events: find the deepest widget at the mouse position
        let target_id = if matches!(
            mouse.kind,
            MouseEventKind::ScrollUp
                | MouseEventKind::ScrollDown
                | MouseEventKind::ScrollLeft
                | MouseEventKind::ScrollRight
        ) {
            // Find scrollview container
            match self.root.find_scrollview_at(mouse.column, mouse.row) {
                Some(id) => id,
                None => return,
            }
        } else {
            // Find deepest widget
            match self.root.find_widget_at(mouse.column, mouse.row) {
                Some(id) => id,
                None => return,
            }
        };

        // Check if Alt is pressed (for horizontal scroll)
        let alt_pressed = mouse.modifiers.contains(KeyModifiers::ALT);

        // Build event context
        let ctx = {
            let ctx = EventContext::new(event_type.clone(), &target_id)
                .with_mouse(mouse.column, mouse.row);
            match mouse.kind {
                MouseEventKind::ScrollUp => ctx.with_scroll(1),
                MouseEventKind::ScrollDown => ctx.with_scroll(-1),
                _ => ctx,
            }
        };

        // For scroll events, handle scroll state first, then bubble
        if matches!(
            mouse.kind,
            MouseEventKind::ScrollUp
                | MouseEventKind::ScrollDown
                | MouseEventKind::ScrollLeft
                | MouseEventKind::ScrollRight
        ) {
            let (delta_x, delta_y) = match mouse.kind {
                MouseEventKind::ScrollLeft => (-1, 0),
                MouseEventKind::ScrollRight => (1, 0),
                MouseEventKind::ScrollUp if alt_pressed => (-1, 0),
                MouseEventKind::ScrollDown if alt_pressed => (1, 0),
                MouseEventKind::ScrollUp => (0, -1),
                MouseEventKind::ScrollDown => (0, 1),
                _ => (0, 0),
            };

            // Handle scroll on the target node
            if let Some(node) = self.root.find_child_mut(&target_id) {
                node.handle_scroll(delta_x, delta_y);
            }
        }

        // Trigger event with bubbling
        self.root.trigger_event_with_bubble(&event_type, ctx);
    }

    /// Handle a UI message from the framework.
    fn handle_ui_msg(&mut self, msg: UiMessage) {
        match msg {
            UiMessage::AddWidget {
                parent_id,
                id,
                widget,
                style,
            } => {
                self.root.add_widget_box(&parent_id, id, widget, style);
            }
            UiMessage::AddContainer {
                parent_id,
                id,
                style,
            } => {
                self.root.add_container(&parent_id, id, style);
            }
            UiMessage::RemoveWidget(id) => {
                self.root.remove_child(&id);
            }
            UiMessage::UpdateWidget { id, widget } => {
                self.root.update_widget_box(&id, widget);
            }
            UiMessage::UpdateStyle { id, style } => {
                self.root.update_style(&id, style);
            }
            UiMessage::AddEventListener {
                target_id,
                event_type,
                listener,
                listener_id,
            } => {
                self.root
                    .add_event_listener(&target_id, event_type, listener, listener_id);
            }
            UiMessage::RemoveEventListener { listener_id } => {
                self.root.remove_event_listener(listener_id);
            }
            UiMessage::AddGlobalListener {
                event_type,
                listener,
                listener_id,
            } => {
                self.global_listeners
                    .entry(event_type)
                    .or_insert_with(Vec::new)
                    .push((listener_id, listener));
            }
            UiMessage::ToggleMouseCapture => {
                self.mouse_capture_enabled = !self.mouse_capture_enabled;
                if self.mouse_capture_enabled {
                    let _ = std::io::stdout().execute(EnableMouseCapture);
                } else {
                    let _ = std::io::stdout().execute(DisableMouseCapture);
                }
            }
            UiMessage::WidgetMessage { id, message } => {
                // Widget-specific message: let the widget handle it
                if let Some(node) = self.root.find_child_mut(&id) {
                    if let Some(widget) = &mut node.widget {
                        message.apply(&mut **widget);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventListener;
    use crate::style::Style;
    use crate::widget::Text;
    use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    struct MockEventSource {
        events: Vec<CrosstermEvent>,
        index: usize,
    }

    impl MockEventSource {
        fn new(events: Vec<CrosstermEvent>) -> Self {
            MockEventSource { events, index: 0 }
        }
    }

    impl EventSource for MockEventSource {
        fn poll(&mut self, _timeout: Duration) -> io::Result<bool> {
            Ok(self.index < self.events.len())
        }
        fn read(&mut self) -> io::Result<CrosstermEvent> {
            if self.index < self.events.len() {
                let event = self.events[self.index].clone();
                self.index += 1;
                Ok(event)
            } else {
                Err(io::Error::new(io::ErrorKind::WouldBlock, "no more events"))
            }
        }
    }

    #[test]
    fn test_new() {
        let rl = RenderLoop::new();
        assert_eq!(rl.root.id, "root");
        assert!(rl.focused_id.is_none());
        assert!(rl.global_listeners.is_empty());
    }

    #[test]
    fn test_add_widget() {
        let mut rl = RenderLoop::new();
        rl.handle_ui_msg(UiMessage::AddWidget {
            parent_id: "root".into(),
            id: "w1".into(),
            widget: Box::new(Text::new("hello")),
            style: Style::new(),
        });
        assert!(rl.root.find_child("w1").is_some());
    }

    #[test]
    fn test_add_and_remove_widget() {
        let mut rl = RenderLoop::new();
        rl.handle_ui_msg(UiMessage::AddWidget {
            parent_id: "root".into(),
            id: "w1".into(),
            widget: Box::new(Text::new("hello")),
            style: Style::new(),
        });
        assert_eq!(rl.root.children.len(), 1);
        rl.handle_ui_msg(UiMessage::RemoveWidget("w1".into()));
        assert_eq!(rl.root.children.len(), 0);
    }

    #[test]
    fn test_add_container() {
        let mut rl = RenderLoop::new();
        rl.handle_ui_msg(UiMessage::AddContainer {
            parent_id: "root".into(),
            id: "c1".into(),
            style: Style::new().column(),
        });
        let node = rl.root.find_child("c1");
        assert!(node.is_some());
        assert!(node.unwrap().widget.is_none());
    }

    #[test]
    fn test_update_style() {
        let mut rl = RenderLoop::new();
        rl.handle_ui_msg(UiMessage::AddWidget {
            parent_id: "root".into(),
            id: "w1".into(),
            widget: Box::new(Text::new("hello")),
            style: Style::new(),
        });
        rl.handle_ui_msg(UiMessage::UpdateStyle {
            id: "w1".into(),
            style: Style::new().bg_color(crate::style::Color::Red),
        });
        let node = rl.root.find_child("w1").unwrap();
        assert_eq!(node.style.bg_color, Some(crate::style::Color::Red));
    }

    #[test]
    fn test_global_listener_fires() {
        let mut rl = RenderLoop::new();
        let fired = Arc::new(AtomicBool::new(false));
        let f = fired.clone();
        let listener: EventListener = Arc::new(move |_| {
            f.store(true, Ordering::SeqCst);
        });

        rl.handle_ui_msg(UiMessage::AddGlobalListener {
            event_type: EventType::KeyPress(KeyCode::Enter),
            listener,
            listener_id: ListenerId::new(),
        });

        rl.trigger_global_listeners(
            &EventType::KeyPress(KeyCode::Enter),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
        assert!(fired.load(Ordering::SeqCst));
    }

    #[test]
    fn test_event_listener_on_widget() {
        let mut rl = RenderLoop::new();
        rl.handle_ui_msg(UiMessage::AddWidget {
            parent_id: "root".into(),
            id: "w1".into(),
            widget: Box::new(Text::new("hello")),
            style: Style::new(),
        });

        // Inject a Click event listener onto "w1" via handle_ui_msg
        let fired = Arc::new(AtomicBool::new(false));
        let f = fired.clone();
        rl.handle_ui_msg(UiMessage::AddEventListener {
            target_id: "w1".into(),
            event_type: EventType::Click,
            listener: Arc::new(move |_| {
                f.store(true, Ordering::SeqCst);
            }),
            listener_id: ListenerId::new(),
        });

        // Simulate the exact block you selected: find widget at coords and trigger click
        if rl.root.find_child_mut("w1").is_some() {
            let ctx = EventContext::new(EventType::Click, "w1").with_mouse(0, 0);
            rl.root.trigger_event_with_bubble(&EventType::Click, ctx);
        }
        assert!(fired.load(Ordering::SeqCst));
    }

    #[test]
    fn test_dispatch_hover_at_empty_area_no_panic() {
        let mut rl = RenderLoop::new();
        rl.dispatch_mouse_event(crossterm::event::MouseEvent {
            kind: MouseEventKind::Moved,
            column: 9999,
            row: 9999,
            modifiers: KeyModifiers::NONE,
        });
    }

    #[test]
    fn test_remove_nonexistent_no_panic() {
        let mut rl = RenderLoop::new();
        rl.handle_ui_msg(UiMessage::RemoveWidget("nope".into()));
    }

    #[test]
    fn test_widget_message_to_nonexistent_no_panic() {
        let mut rl = RenderLoop::new();
        rl.handle_ui_msg(UiMessage::WidgetMessage {
            id: "nope".into(),
            message: Box::new(crate::widget::text::TextMessage::SetContent("x".into())),
        });
    }

    #[tokio::test]
    async fn test_run_exits_on_closed_channel() {
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let terminal = Terminal::new(backend).unwrap();
        let (ui_tx, ui_rx) = mpsc::channel(100);
        let (_event_tx, _event_rx) = mpsc::channel(100);
        let source = MockEventSource::new(vec![]);

        let handle = tokio::spawn(async move {
            RenderLoop::run(terminal, ui_rx, source, _event_tx)
                .await
                .ok();
        });

        // Give the loop a chance to start and poll once
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Drop sender to close the channel
        drop(ui_tx);

        // Should exit within timeout
        tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("run() did not exit after channel closed")
            .ok();
    }

    #[tokio::test]
    async fn test_run_forwards_key_event() {
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let terminal = Terminal::new(backend).unwrap();
        let (ui_tx, ui_rx) = mpsc::channel(100);
        let (event_tx, mut event_rx) = mpsc::channel(100);

        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('a'),
            KeyModifiers::NONE,
        );
        let source = MockEventSource::new(vec![CrosstermEvent::Key(key)]);

        tokio::spawn(async move {
            RenderLoop::run(terminal, ui_rx, source, event_tx)
                .await
                .ok();
        });

        // Wait for event to be processed
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Drop sender so loop exits
        drop(ui_tx);
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Verify key was forwarded
        let received = event_rx.try_recv().unwrap();
        match received {
            Event::Key(k) => assert_eq!(k.code, crossterm::event::KeyCode::Char('a')),
            _ => panic!("expected Key event"),
        }
    }

    #[tokio::test]
    async fn test_run_forwards_mouse_event() {
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let terminal = Terminal::new(backend).unwrap();
        let (ui_tx, ui_rx) = mpsc::channel(100);
        let (event_tx, mut event_rx) = mpsc::channel(100);

        let mouse = crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 10,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        let source = MockEventSource::new(vec![CrosstermEvent::Mouse(mouse)]);

        tokio::spawn(async move {
            RenderLoop::run(terminal, ui_rx, source, event_tx)
                .await
                .ok();
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        drop(ui_tx);
        tokio::time::sleep(Duration::from_millis(50)).await;

        let received = event_rx.try_recv().unwrap();
        match received {
            Event::Mouse(m) => assert_eq!(m.column, 10),
            _ => panic!("expected Mouse event"),
        }
    }

    #[tokio::test]
    async fn test_run_forwards_resize_event() {
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let terminal = Terminal::new(backend).unwrap();
        let (ui_tx, ui_rx) = mpsc::channel(100);
        let (event_tx, mut event_rx) = mpsc::channel(100);

        let source = MockEventSource::new(vec![CrosstermEvent::Resize(100, 30)]);

        tokio::spawn(async move {
            RenderLoop::run(terminal, ui_rx, source, event_tx)
                .await
                .ok();
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        drop(ui_tx);
        tokio::time::sleep(Duration::from_millis(50)).await;

        let received = event_rx.try_recv().unwrap();
        match received {
            Event::Resize(w, h) => {
                assert_eq!(w, 100);
                assert_eq!(h, 30);
            }
            _ => panic!("expected Resize event"),
        }
    }

    #[tokio::test]
    async fn test_run_global_listener_fires() {
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let terminal = Terminal::new(backend).unwrap();
        let (ui_tx, ui_rx) = mpsc::channel(100);
        let (_event_tx, _event_rx) = mpsc::channel(100);

        let fired = Arc::new(AtomicBool::new(false));
        let f = fired.clone();
        let listener: EventListener = Arc::new(move |_| {
            f.store(true, Ordering::SeqCst);
        });
        let listener_id = ListenerId::new();

        // Send the AddGlobalListener before spawning run()
        ui_tx
            .try_send(UiMessage::AddGlobalListener {
                event_type: EventType::KeyPress(KeyCode::Enter),
                listener,
                listener_id,
            })
            .unwrap();

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let source = MockEventSource::new(vec![CrosstermEvent::Key(key)]);

        tokio::spawn(async move {
            RenderLoop::run(terminal, ui_rx, source, _event_tx)
                .await
                .ok();
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        drop(ui_tx);
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(fired.load(Ordering::SeqCst));
    }
}
