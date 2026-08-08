// crates/mediadl-tui/examples/field_playground.rs
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols::line::HORIZONTAL,
    text::Line,
    widgets::{Block, BorderType, Paragraph},
};

struct Field {
    label: &'static str,
    text: String,
    cursor: usize,
}

impl Field {
    fn new(label: &'static str) -> Self {
        Self {
            label,
            text: String::new(),
            cursor: 0,
        }
    }
    fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += 1;
    }
    fn delete_char(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        self.text.remove(self.cursor);
    }
    fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }
    fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += 1;
        }
    }
}

fn padding_lines(offset: usize) -> String {
    HORIZONTAL.repeat(offset)
}

fn centered_row(area: Rect, width: u16, height: u16) -> Rect {
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    area
}

/// Static placeholder — unchanged from ui.rs.
fn field_display(text: &str, width: usize) -> String {
    let content_width = width.saturating_sub(1);
    let char_count = text.chars().count();
    let content = if char_count > content_width {
        let keep = content_width.saturating_sub(3);
        let s: String = text.chars().take(keep).collect();
        // s.push_str("...");
        s
    } else {
        text.to_string()
    };
    format!("{:<width$}", content, width = content_width)
}

/// Live-typed version. Scrolls to keep the cursor visible; shows "..." only
/// when there's hidden text *past* the visible window — i.e. the cursor has
/// been moved away from the true end of the text, not while just typing at
/// the end.
/// Live-typed version. Scrolls to keep the cursor visible, no truncation marker.
fn field_display_editable(text: &str, cursor: usize, width: usize) -> (String, usize) {
    let content_width = width.saturating_sub(1);
    if content_width == 0 {
        return (String::new(), 0);
    }
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    let scroll_start = if cursor >= content_width {
        cursor - content_width + 1
    } else {
        0
    };
    let scroll_end = (scroll_start + content_width).min(len);
    let visible: String = chars[scroll_start..scroll_end].iter().collect();

    let cursor_col = cursor.saturating_sub(scroll_start);
    (
        format!("{:<width$}", visible, width = content_width),
        cursor_col,
    )
}

fn draw(fields: &[Field; 3], active: usize, frame: &mut Frame) {
    let area = frame.area();
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(Line::from(format!("{} Download ", padding_lines(3))))
        .title_bottom(Line::from(format!(
            "{} tab:next {} shift+tab:prev {} esc:quit",
            padding_lines(3),
            padding_lines(2),
            padding_lines(2),
        )));
    let inner = area.inner(Margin::new(1, 1));

    let widths = [10u16, 12, 10];
    let total_width: u16 = widths.iter().sum::<u16>() + 2 + 4;
    let row = centered_row(inner, total_width, 1);

    let placeholder_style = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::UNDERLINED);
    let filled_style = Style::default().add_modifier(Modifier::UNDERLINED);

    let [creator, sep1, collection, sep2, url] = Layout::horizontal([
        Constraint::Length(widths[0]),
        Constraint::Length(1),
        Constraint::Length(widths[1]),
        Constraint::Length(1),
        Constraint::Length(widths[2]),
    ])
    .spacing(1)
    .areas(row);

    let rects = [creator, collection, url];
    let mut cursor_target: Option<(u16, u16)> = None;

    for (i, (field, rect)) in fields.iter().zip(rects.iter()).enumerate() {
        if field.text.is_empty() && i != active {
            frame.render_widget(
                Paragraph::new(field_display(field.label, widths[i] as usize))
                    .style(placeholder_style),
                *rect,
            );
            continue;
        }

        let (display, cursor_col) =
            field_display_editable(&field.text, field.cursor, widths[i] as usize);
        frame.render_widget(Paragraph::new(display).style(filled_style), *rect);

        if i == active {
            cursor_target = Some((rect.x + cursor_col as u16, rect.y));
        }
    }

    frame.render_widget(Paragraph::new("/"), sep1);
    frame.render_widget(Paragraph::new("/"), sep2);

    if let Some((x, y)) = cursor_target {
        frame.buffer_mut().set_style(
            Rect::new(x, y, 1, 1),
            Style::default().add_modifier(Modifier::REVERSED),
        );
    }

    frame.render_widget(block, area);
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let mut fields = [
        Field::new("Creator"),
        Field::new("Collection"),
        Field::new("Url"),
    ];
    let mut active = 0usize;

    loop {
        terminal.draw(|frame| draw(&fields, active, frame))?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    active = (active + fields.len() - 1) % fields.len();
                }
                KeyCode::BackTab => {
                    active = (active + fields.len() - 1) % fields.len();
                }
                KeyCode::Tab => {
                    active = (active + 1) % fields.len();
                }
                KeyCode::Char(c) => fields[active].insert_char(c),
                KeyCode::Backspace => fields[active].delete_char(),
                KeyCode::Left => fields[active].move_left(),
                KeyCode::Right => fields[active].move_right(),
                _ => {}
            }
        }
    }

    ratatui::restore();
    Ok(())
}
