// ui.rs
use crate::app::App;
use crate::states::download::{DownloadState, DownloadType};
// use crate::states::Screen;
use ratatui::Frame;
use ratatui::{
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::line::HORIZONTAL,
    text::{Line, Span},
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

        let title = Line::from_iter([Span::from("mediadl").bold()]);
        frame.render_widget(title.centered(), top);

        self.render_menu(frame, left);
        self.render_download(frame, download_panel);
        self.render_output(frame, output_panel);
    }

    fn render_menu(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
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
                item.style(Style::default().bold().blue())
            } else {
                item
            }
        });

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
    // ↑ ↗ → ↘ ↓ ↙ ← ↖

    fn render_download(&self, frame: &mut Frame, area: Rect) {
        let fields = self.download.field_items();

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from(format!("{} Download ", Self::padding_lines(3))))
            .title_bottom(Line::from(format!(
                "{} →:tab {} ←:shift+tab {} i:edit {} download:enter",
                Self::padding_lines(3),
                Self::padding_lines(2),
                Self::padding_lines(2),
                Self::padding_lines(2),
            )));
        let inner = area.inner(Margin::new(1, 1));
        let widths = [10u16, 12, 10];
        let separator_columns = 2;
        let spacing_gaps = 4;
        let total_width: u16 = widths.iter().sum::<u16>() + separator_columns + spacing_gaps;
        let row = Self::centered_row(inner, total_width, 1);

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

        let [creator, sep1, collection, sep2, url] = Layout::horizontal([
            Constraint::Length(widths[0]),
            Constraint::Length(1),
            Constraint::Length(widths[1]),
            Constraint::Length(1),
            Constraint::Length(widths[2]),
        ])
        .spacing(1)
        .areas(row);

        let [
            (creator_name, creator_value, creator_active),
            (collection_name, collection_value, collection_active),
            (url_name, url_value, url_active),
        ] = fields.as_slice()
        else {
            return;
        };

        let creator_style = Self::field_style(*creator_active);
        let collection_style = Self::field_style(*collection_active);
        let url_style = Self::field_style(*url_active);

        if self.download.is_editing() && *creator_active {
            Self::render_editable_field(
                frame,
                creator,
                creator_value,
                self.download.active_cursor(),
                widths[0] as usize,
            );
        } else {
            frame.render_widget(
                Paragraph::new(if creator_value.is_empty() {
                    format!("{}", creator_name)
                } else {
                    Self::field_display(creator_value, widths[0] as usize)
                })
                .style(creator_style),
                creator,
            );
        }

        frame.render_widget(Paragraph::new("/"), sep1);

        if self.download.is_editing() && *collection_active {
            Self::render_editable_field(
                frame,
                creator,
                creator_value,
                self.download.active_cursor(),
                widths[0] as usize,
            );
        } else {
            frame.render_widget(
                Paragraph::new(if collection_value.is_empty() {
                    format!("{}", collection_name)
                } else {
                    Self::field_display(collection_value, widths[1] as usize)
                })
                .style(collection_style),
                collection,
            );
        }

        frame.render_widget(Paragraph::new("/"), sep2);

        if self.download.is_editing() && *url_active {
            Self::render_editable_field(
                frame,
                url,
                url_value,
                self.download.active_cursor(),
                widths[2] as usize,
            );
        } else {
            frame.render_widget(
                Paragraph::new(if url_value.is_empty() {
                    format!("{}", url_name)
                } else {
                    Self::field_display(url_value, widths[2] as usize)
                })
                .style(url_style),
                url,
            );
        }
        frame.render_widget(block, area);
    }

    fn render_output(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Line::from(format!("{} Output ", Self::padding_lines(3))))
            .title_bottom(Line::from(format!(
                "{} q:ctrl+q {} options:C ",
                Self::padding_lines(3),
                Self::padding_lines(2)
            )));
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

    fn field_style(active: bool) -> Style {
        if active {
            Style::default().bold().blue()
        } else {
            Style::default().dark_gray()
        }
    }

    // only during normal mode
    fn field_display(text: &str, width: usize) -> String {
        // last column for /
        // TODO: scrolling
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

        // ensures that / is always at the end
        // pads out the values
        format!("{:<width$}", content, width = content_width)
    }

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

    fn render_editable_field(
        frame: &mut Frame,
        area: Rect,
        text: &str,
        cursor: usize,
        width: usize,
    ) {
        let (visible, cursor_col) = Self::field_display_editable(text, cursor, width);

        let paragraph = Paragraph::new(visible).style(Style::default().bold().blue());

        frame.render_widget(paragraph, area);

        frame.set_cursor_position((area.x + cursor_col as u16, area.y));
    }
}
