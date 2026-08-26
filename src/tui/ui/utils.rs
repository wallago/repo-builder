use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Stylize},
    text::Span,
    widgets::{Block, Paragraph},
};

use crate::app::tool::category::Category;

/// A colored pill/badge: ` TEXT ` with a background.
pub(crate) fn badge(category: Category) -> Span<'static> {
    format!("|{category}|")
        .bg(category.color())
        .fg(Color::Black)
        .bold()
}

/// A single bordered text field.
pub(crate) fn render_input(
    frame: &mut Frame,
    rect: Rect,
    title: &str,
    value: &str,
    focused: bool,
    is_editable: bool,
    cursor_active: bool,
) {
    let title = format!(" {title} ").fg(Color::Gray).bold();
    let mut block = Block::bordered().title(title);
    if focused && !is_editable {
        block = block.border_style(Color::Yellow);
    } else if focused && is_editable {
        block = block.border_style(Color::Green);
    }
    let cursor = if is_editable && focused && cursor_active {
        "█"
    } else {
        " "
    };
    let body = format!("{value}{cursor}");
    frame.render_widget(Paragraph::new(body).block(block), rect);
}
