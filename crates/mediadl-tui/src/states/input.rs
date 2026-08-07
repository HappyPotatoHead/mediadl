use crate::traits::Named;

#[derive(Debug, Default)]
pub struct TextInput {
    text: String,
    cursor: usize,
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum InputField {
    #[default]
    Creator,
    Collection,
    Url,
    Type,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntryType {
    Video,
    Audio,
}

#[derive(Debug, Default, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Edit,
}

impl Named for InputField {
    fn name(&self) -> &'static str {
        match self {
            InputField::Creator => "Creator",
            InputField::Collection => "Collection",
            InputField::Url => "Url",
            InputField::Type => "Type",
        }
    }
}

impl EntryType {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_lowercase().as_str() {
            "video" | "v" => Ok(Self::Video),
            "audio" | "a" => Ok(Self::Audio),
            other => Err(format!(
                "type must be \"video\" or \"audio\", got \"{other}\""
            )),
        }
    }
}

impl TextInput {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.cursor = self.text.len();
    }

    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let previous = self.text[..self.cursor]
            .char_indices()
            .next_back()
            .map(|(index, ch)| (index, ch.len_utf8()));

        if let Some((index, len)) = previous {
            self.text.drain(index..index + len);
            self.cursor = index;
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor == 0 {
            return;
        }

        self.cursor = self.text[..self.cursor]
            .char_indices()
            .next_back()
            .map(|(index, _)| index)
            .unwrap_or(0);
    }

    pub fn move_right(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }
        self.cursor += self.text[self.cursor..].chars().next().unwrap().len_utf8();
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.text.len();
    }
}

#[cfg(test)]
mod tests {
    use super::TextInput;

    #[test]
    fn text_input_handles_unicode() {
        let mut input = TextInput::default();

        input.set_text("héllo");
        input.move_left();
        input.insert_char('X');

        assert_eq!(input.text(), "héllXo"); // depending on cursor semantics
    }

    #[test]
    fn text_input_handles_deletion() {
        let mut input = TextInput::default();

        input.set_text("héllo");
        input.delete_char();

        assert_eq!(input.text(), "héll");
    }

    #[test]
    fn text_input_moves_across_unicode_character() {
        let mut input = TextInput::default();

        input.set_text("héllo");
        input.move_home();

        input.move_right();
        assert_eq!(input.cursor_position(), 1);

        input.move_right();
        assert_eq!(input.cursor_position(), 3);
    }
}
