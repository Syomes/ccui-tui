use ccui::{
    Container, Ui,
    event::{Event, EventType},
    style::Style,
    widget::Text,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Ui::run()?;

    // Add widgets
    doc.add_widget("title", Text::new("Hello"))?;

    // Add container with children
    let mut row = doc.add_container("row", Style::new().row())?;
    row.add_widget("btn", Text::new("Click me"))?;

    // Add event listener
    doc.add_event_listener("btn", EventType::Click, |ctx| {
        println!("Clicked at ({:?}, {:?})", ctx.mouse_x, ctx.mouse_y);
    })?;

    // Handle events
    while let Some(event) = doc.event_receiver().recv().await {
        if let Event::Key(key) = event {
            if key.code == crossterm::event::KeyCode::Char('q') {
                break;
            }
        }
    }

    Ok(())
}
