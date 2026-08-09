// ui.rs
use crate::app::App;
use crate::states::download::DownloadType;
// use crate::states::Screen;
use ratatui::Frame;
use ratatui::{
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::line::HORIZONTAL,
    text::Line,
    widgets::{Block, BorderType, List, ListItem, Padding, Paragraph},
};

/*
*keyboard
   ↓
App handles event
   ↓
DownloadState changes
   ↓
render()
   ↓
UI reads DownloadState
   ↓
terminal changes
*/

// creating ui
// render -> "head"
// download_render -> download menus / download input // output
// config_render -> editor / control

// anything related to drawing is here

impl App {
    pub fn render(&self, frame: &mut Frame) {
        // Length(1) sets the height
        // Fill(1) tells the second chunk to absorb all remain spaces
        let vertical_main =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(0);
        // creates two columns, each takes up 33% of the available space
        let horizontal_main =
            Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).spacing(1);
        let vertical_right =
            Layout::vertical([Constraint::Length(5), Constraint::Fill(1)]).spacing(0);

        let [top, main] = frame.area().layout(&vertical_main);
        let [left, right] = main.layout(&horizontal_main);
        let [download_panel, output_panel] = right.layout(&vertical_right);
        let title = Line::from("mediadl")
            .bold()
            .fg("#CECDC3".parse::<Color>().unwrap());
        // let title = Line::from_iter([Span::from("mediadl").bold().fg("#CECDC3".parse().unwrap())]);
        frame.render_widget(title.centered(), top);

        self.render_menu(frame, left);
        self.render_download(frame, download_panel);
        self.render_output(frame, output_panel);
    }

    fn render_menu(&self, frame: &mut Frame, area: Rect) {
        let focused = self.download.is_menu_focus();
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Self::focused_style(focused))
            .padding(Padding::new(2, 0, 1, 0))
            .title(Line::from(format!("{} Menu ", Self::padding_lines(3))))
            .title_bottom(Line::from(format!(
                "{} ↓:j {} ↑:k {}",
                Self::padding_lines(3),
                Self::padding_lines(1),
                Self::padding_lines(1)
            )));

        let items = vec![
            ("Video", DownloadType::Video),
            ("Audio", DownloadType::Audio),
            ("Batch", DownloadType::Batch),
        ];
        let items = items.into_iter().map(|(name, mode)| {
            let item = ListItem::new(name);

            if self.download.get_mode() == &mode {
                item.style(Style::default().bold().fg("#66A0C8".parse().unwrap()))
            } else {
                item.dark_gray()
            }
        });

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
    // ↑ ↗ → ↘ ↓ ↙ ← ↖

    fn render_download(&self, frame: &mut Frame, area: Rect) {
        let focused = self.download.is_input_focus();
        // "", value, boolean
        let fields = self.download.field_items();

        let inner = area.inner(Margin::new(1, 1));

        let row_width = (inner.width * 80) / 100;
        let row = Self::centered_row(inner, row_width, 1);

        let field_constraints = match fields.len() {
            3 => vec![
                Constraint::Percentage(20),
                Constraint::Length(1),
                Constraint::Percentage(40),
                Constraint::Length(1),
                Constraint::Percentage(40),
            ],
            2 => vec![
                Constraint::Percentage(70),
                Constraint::Length(1),
                Constraint::Percentage(30),
            ],
            _ => return,
        };

        let areas = Layout::horizontal(field_constraints).spacing(1).split(row);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Self::focused_style(focused))
            .title(Line::from(format!("{} Download ", Self::padding_lines(3))))
            .title_bottom(Line::from(format!(
                "{} ←:j {} →:k {} edit:i {} clear:c ",
                Self::padding_lines(3),
                Self::padding_lines(1),
                Self::padding_lines(1),
                Self::padding_lines(1),
            )))
            .title_top(
                Line::from(format!(
                    " normal:esc {} install:enter {}",
                    Self::padding_lines(1),
                    Self::padding_lines(3)
                ))
                .right_aligned(),
            );

        for (index, (name, value, active)) in fields.iter().enumerate() {
            let area = areas[index * 2];

            if self.download.is_editing() && *active {
                Self::render_editable_field(
                    frame,
                    area,
                    value,
                    self.download.active_cursor(),
                    area.width as usize,
                );
            } else {
                let text = if value.is_empty() {
                    name.to_string()
                } else {
                    Self::field_display(value, area.width as usize)
                };

                let style = Self::field_style(&self, *active);

                frame.render_widget(Paragraph::new(text).style(style), area);
            }
        }

        for index in 0..fields.len().saturating_sub(1) {
            let separator_area = areas[index * 2 + 1];

            frame.render_widget(Paragraph::new("/"), separator_area);
        }

        frame.render_widget(block, area);
    }

    fn render_output(&self, frame: &mut Frame, area: Rect) {
        let focused = self.download.is_output_focus();
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Self::focused_style(focused))
            .title(Line::from(format!("{} Output ", Self::padding_lines(3))))
            .title_bottom(Line::from(format!(
                "{} q:ctrl+q {} options:C {} →:tab {} ←:shift+tab ",
                Self::padding_lines(3),
                Self::padding_lines(1),
                Self::padding_lines(1),
                Self::padding_lines(1)
            )));
        let inner = area.inner(Margin::new(1, 1));
        let width = inner.width as usize;
        if width == 0 {
            return;
        }

        let mut formatted_lines: Vec<Line> = Vec::new();
        for raw_line in self.output.lines() {
            let trimmed = raw_line.trim();

            if trimmed.is_empty() {
                formatted_lines.push(Line::from(""));
                continue;
            }

            let clean_str = trimmed
                .strip_prefix("[download]")
                .or_else(|| trimmed.strip_prefix("[ExtractAudio]"))
                .or_else(|| trimmed.strip_prefix("[EmbedThumbnail]"))
                .unwrap_or(trimmed)
                .trim();

            let line_style =
                if clean_str.starts_with("Error:") || clean_str.starts_with("Download failed") {
                    Style::default().fg(Color::Red).bold()
                } else if clean_str.starts_with("Starting") {
                    Style::default().fg(Color::Cyan)
                } else if clean_str.starts_with("Download finished")
                    || clean_str.starts_with("Finished")
                {
                    Style::default().fg(Color::Green).bold()
                } else {
                    Style::default().dark_gray()
                };

            let chars: Vec<char> = clean_str.chars().collect();
            if chars.is_empty() {
                formatted_lines.push(Line::from("").style(line_style));
            } else {
                for chunk in chars.chunks(width) {
                    let chunk_str: String = chunk.iter().collect();
                    formatted_lines.push(Line::from(chunk_str).style(line_style));
                }
            }
            // formatted_lines.push(Line::from(clean_str).style(line_style));
        }
        let total_rows = formatted_lines.len();
        let visible_rows = inner.height as usize;
        let max_scroll = total_rows.saturating_sub(visible_rows);

        self.output.set_last_max_scroll(max_scroll);

        let scroll = if self.output.follow_tail() {
            max_scroll
        } else {
            self.output.scroll_offset().min(max_scroll)
        };
        let paragraph = Paragraph::new(formatted_lines).scroll((scroll as u16, 0));

        frame.render_widget(paragraph, inner);
        frame.render_widget(block, area);
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

    fn field_style(&self, active: bool) -> Style {
        let style = if active && self.download.is_input_focus() {
            Style::default().bold().fg("#66A0C8".parse().unwrap())
        } else {
            Style::default().dark_gray()
        };
        style.underlined()
    }

    // only during normal mode
    fn field_display(text: &str, width: usize) -> String {
        let content_width = width.saturating_sub(1);
        let char_count = text.chars().count();

        let content = if char_count > content_width {
            // content_width - 3
            let keep = content_width.saturating_sub(3);
            let mut s: String = text.chars().take(keep).collect();
            // adds ... right after the values that were kept
            s.push_str("...");
            s
        } else {
            text.to_string()
        };

        // pads out the values
        format!("{:<width$}", content, width = content_width)
    }

    fn field_display_editable(text: &str, cursor: usize, width: usize) -> (String, usize) {
        // let content_width = width.saturating_sub(1);
        if width == 0 {
            return (String::new(), 0);
        }
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        let scroll_start = if cursor >= width {
            cursor - width + 1
        } else {
            0
        };
        let scroll_end = (scroll_start + width).min(len);
        let visible: String = chars[scroll_start..scroll_end].iter().collect();

        let cursor_col = cursor.saturating_sub(scroll_start);
        (format!("{:<width$}", visible, width = width), cursor_col)
    }

    fn render_editable_field(
        frame: &mut Frame,
        area: Rect,
        text: &str,
        cursor: usize,
        width: usize,
    ) {
        let (visible, cursor_col) = Self::field_display_editable(text, cursor, width);

        let paragraph =
            Paragraph::new(visible).style(Style::default().bold().fg("#66A0C8".parse().unwrap()));

        frame.render_widget(paragraph, area);

        frame.set_cursor_position((area.x + cursor_col as u16, area.y));
    }

    fn focused_style(focused: bool) -> Style {
        let border_colour = if focused { "#4385BE" } else { "#6F6E69" };
        if focused {
            Style::default().fg(border_colour.parse().unwrap())
        } else {
            Style::default().fg(border_colour.parse().unwrap())
        }
    }
}

// DUMP
//
// Paragraph::new(text).style(style);
// let items = fields.iter().map(|(name, value, active)| {
//     let text = if value.is_empty() {
//         format!("<{}>", name)
//     } else {
//         value.to_string()
//     };
//
//     let style = if *active {
//         Style::default().bold().blue()
//     } else {
//         Style::default()
//     };
//
//     Paragraph::new(text).style(style)
// });

// if fields.len() == 3 {
//     let [creator, sep1, collection, sep2, url] = Layout::horizontal([
//         Constraint::Length(widths[0]),
//         Constraint::Length(1),
//         Constraint::Length(widths[1]),
//         Constraint::Length(1),
//         Constraint::Length(widths[2]),
//     ])
//     .spacing(1)
//     .areas(row);
//
//     let [
//         (creator_name, creator_value, creator_active),
//         (collection_name, collection_value, collection_active),
//         (url_name, url_value, url_active),
//     ] = fields.as_slice()
//     else {
//         return;
//     };
//
//     let creator_style = Self::field_style(*creator_active);
//     let collection_style = Self::field_style(*collection_active);
//     let url_style = Self::field_style(*url_active);
//
//     if self.download.is_editing() && *creator_active {
//         Self::render_editable_field(
//             frame,
//             creator,
//             creator_value,
//             self.download.active_cursor(),
//             widths[0] as usize,
//         );
//     } else {
//         frame.render_widget(
//             Paragraph::new(if creator_value.is_empty() {
//                 format!("{}", creator_name)
//             } else {
//                 Self::field_display(creator_value, widths[0] as usize)
//             })
//             .style(creator_style),
//             creator,
//         );
//     }
//
//     frame.render_widget(Paragraph::new("/"), sep1);
//
//     if self.download.is_editing() && *collection_active {
//         Self::render_editable_field(
//             frame,
//             collection,
//             collection_value,
//             self.download.active_cursor(),
//             widths[0] as usize,
//         );
//     } else {
//         frame.render_widget(
//             Paragraph::new(if collection_value.is_empty() {
//                 format!("{}", collection_name)
//             } else {
//                 Self::field_display(collection_value, widths[1] as usize)
//             })
//             .style(collection_style),
//             collection,
//         );
//     }
//
//     frame.render_widget(Paragraph::new("/"), sep2);
//
//     if self.download.is_editing() && *url_active {
//         Self::render_editable_field(
//             frame,
//             url,
//             url_value,
//             self.download.active_cursor(),
//             widths[2] as usize,
//         );
//     } else {
//         frame.render_widget(
//             Paragraph::new(if url_value.is_empty() {
//                 format!("{}", url_name)
//             } else {
//                 Self::field_display(url_value, widths[2] as usize)
//             })
//             .style(url_style),
//             url,
//         );
//     }
// } else {
//     let [url, sep1, kind] = Layout::horizontal([
//         Constraint::Length(widths[2]),
//         Constraint::Length(1),
//         Constraint::Length(widths[0]),
//     ])
//     .spacing(1)
//     .areas(row);
//
//     let [
//         (url_name, url_value, url_active),
//         (kind_name, kind_value, kind_active),
//     ] = fields.as_slice()
//     else {
//         return;
//     };
//
//     let url_style = Self::field_style(*url_active);
//     let kind_style = Self::field_style(*kind_active);
//
//     if self.download.is_editing() && *url_active {
//         Self::render_editable_field(
//             frame,
//             url,
//             url_value,
//             self.download.active_cursor(),
//             widths[2] as usize,
//         );
//     } else {
//         frame.render_widget(
//             Paragraph::new(if url_value.is_empty() {
//                 format!("{}", url_name)
//             } else {
//                 Self::field_display(url_value, widths[2] as usize)
//             })
//             .style(url_style),
//             url,
//         );
//     }
//
//     frame.render_widget(Paragraph::new("/"), sep1);
//
//     if self.download.is_editing() && *kind_active {
//         Self::render_editable_field(
//             frame,
//             kind,
//             kind_value,
//             self.download.active_cursor(),
//             widths[0] as usize,
//         );
//     } else {
//         frame.render_widget(
//             Paragraph::new(if kind_value.is_empty() {
//                 format!("{}", kind_name)
//             } else {
//                 Self::field_display(kind_value, widths[0] as usize)
//             })
//             .style(kind_style),
//             kind,
//         );
//     }
// }
