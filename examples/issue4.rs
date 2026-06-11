use ccui::{
    Container, Ui,
    event::Event,
    style::{BorderType, Color, Overflow, Style},
    widget::Text,
};
use crossterm::event::KeyCode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Ui::run()?;
    let mut win = doc.add_container(
        "window",
        Style::new()
            .column()
            .auto()
            .floating()
            .size(50, 10)
            .border(BorderType::Double),
    )?;
    win.update_style(|s| {
        s.overflow = Overflow::Scroll;
    })?;
    let mut cont =
        win.add_container("cont", Style::new().row().auto().border(BorderType::Plain))?;
    cont.update_style(|s| {
        s.bg_color = Some(Color::Blue);
    })?;
    cont.add_widget("txt", Text::new("text"))?;

    // Keep the application running
    while let Some(event) = doc.event_receiver().recv().await {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') => {
                    break;
                }
                KeyCode::F(12) => {
                    doc.toggle_mouse_capture()?;
                }
                KeyCode::Tab => {}
                _ => {}
            }
        }
    }

    Ok(())
}
