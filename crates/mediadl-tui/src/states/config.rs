use crate::states::input::InputMode;
use crate::states::input::TextInput;
use crate::traits::Cycle;
use crate::traits::Named;
use crate::traits::VerticalNavigation;

use crossterm::event::{KeyCode, KeyEvent};
use mediadl_core::config::{AppConfig, default_config_path};

#[derive(Debug)]
pub struct ConfigState {
    pub config: AppConfig,
    selected: ConfigField,
    input_mode: InputMode,
    edit_buffer: TextInput,
}

#[derive(Debug, Default, PartialEq)]
enum ConfigField {
    #[default]
    DownloadPath,
    AudioFormat,
    VideoFormat,
    VideoQuality,
    AudioThumbnail,
    VideoThumbnail,
    AudioOutputTemplate,
    VideoOutputTemplate,
    Retries,
    MaxParallel,
}

impl ConfigState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            selected: ConfigField::default(),
            input_mode: InputMode::default(),
            edit_buffer: TextInput::default(),
        }
    }

    // think about moving these as traits later
    pub fn is_editing(&self) -> bool {
        self.input_mode == InputMode::Edit
    }
    pub fn begin_edit(&mut self) {
        let current = self.config.get_by_key(self.selected.key());
        self.edit_buffer = TextInput::default();
        self.edit_buffer.set_text(&current);
        self.input_mode = InputMode::Edit;
    }
    pub fn handle_edit_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<Option<String>> {
        if matches!(key_event.code, KeyCode::Esc) {
            self.exit_edit();
            return Ok(None);
        }

        if matches!(key_event.code, KeyCode::Enter) {
            let value = self.edit_buffer.text().to_string();
            return match self.config.set_by_key(self.selected.key(), &value) {
                Ok(()) => {
                    self.exit_edit();
                    Ok(Some(format!("Updated {}", self.selected.name())))
                }
                Err(err) => Ok(Some(format!("Error: {err}"))),
            };
        }

        match key_event.code {
            KeyCode::Char(c) => self.edit_buffer.insert_char(c),
            KeyCode::Backspace => self.edit_buffer.delete_char(),
            KeyCode::Left => self.edit_buffer.move_left(),
            KeyCode::Right => self.edit_buffer.move_right(),
            KeyCode::Home => self.edit_buffer.move_home(),
            KeyCode::End => self.edit_buffer.move_end(),
            _ => {}
        }
        Ok(None)
    }
    pub fn edit_text(&self) -> &str {
        self.edit_buffer.text()
    }

    pub fn edit_cursor(&self) -> usize {
        self.edit_buffer.cursor_position()
    }

    pub fn field_items(&self) -> impl Iterator<Item = (&'static str, String, bool)> + '_ {
        ConfigField::ALL.into_iter().map(move |f| {
            (
                f.name(),
                self.config.get_by_key(f.key()),
                f == self.selected,
            )
        })
    }

    pub fn save(&self) -> Result<(), String> {
        let path = default_config_path()?;
        self.config.save_config_file(path)
    }

    fn exit_edit(&mut self) {
        self.input_mode = InputMode::Normal;
    }
}

impl VerticalNavigation for ConfigState {
    fn move_up(&mut self) {
        self.selected.prev();
    }

    fn move_down(&mut self) {
        self.selected.next();
    }
}

impl ConfigField {
    const ALL: [Self; 10] = [
        Self::DownloadPath,
        Self::AudioFormat,
        Self::VideoFormat,
        Self::VideoQuality,
        Self::AudioThumbnail,
        Self::VideoThumbnail,
        Self::AudioOutputTemplate,
        Self::VideoOutputTemplate,
        Self::Retries,
        Self::MaxParallel,
    ];
    pub fn key(&self) -> &'static str {
        match self {
            Self::DownloadPath => "download_path",
            Self::AudioFormat => "audio_format",
            Self::VideoFormat => "video_format",
            Self::VideoQuality => "video_quality",
            Self::AudioThumbnail => "audio_thumbnail",
            Self::VideoThumbnail => "video_thumbnail",
            Self::AudioOutputTemplate => "audio_output_template",
            Self::VideoOutputTemplate => "video_output_template",
            Self::Retries => "retries",
            Self::MaxParallel => "max_parallel_downloads",
        }
    }
}

impl Cycle for ConfigField {
    fn next(&mut self) {
        *self = match self {
            Self::DownloadPath => Self::AudioFormat,
            Self::AudioFormat => Self::VideoFormat,
            Self::VideoFormat => Self::VideoQuality,
            Self::VideoQuality => Self::AudioThumbnail,
            Self::AudioThumbnail => Self::VideoThumbnail,
            Self::VideoThumbnail => Self::AudioOutputTemplate,
            Self::AudioOutputTemplate => Self::VideoOutputTemplate,
            Self::VideoOutputTemplate => Self::Retries,
            Self::Retries => Self::MaxParallel,
            Self::MaxParallel => Self::DownloadPath,
        };
    }

    fn prev(&mut self) {
        *self = match self {
            Self::DownloadPath => Self::MaxParallel,
            Self::AudioFormat => Self::DownloadPath,
            Self::VideoFormat => Self::AudioFormat,
            Self::VideoQuality => Self::VideoFormat,
            Self::AudioThumbnail => Self::VideoQuality,
            Self::VideoThumbnail => Self::AudioThumbnail,
            Self::AudioOutputTemplate => Self::VideoThumbnail,
            Self::VideoOutputTemplate => Self::AudioOutputTemplate,
            Self::Retries => Self::VideoOutputTemplate,
            Self::MaxParallel => Self::Retries,
        };
    }
}

impl Named for ConfigField {
    fn name(&self) -> &'static str {
        match self {
            Self::DownloadPath => "Download Path",
            Self::AudioFormat => "Audio Format",
            Self::VideoFormat => "Video Format",
            Self::VideoQuality => "Video Quality",
            Self::AudioThumbnail => "Audio Thumbnail",
            Self::VideoThumbnail => "Video Thumbnail",
            Self::AudioOutputTemplate => "Audio Output Template",
            Self::VideoOutputTemplate => "Video Output Template",
            Self::Retries => "Retries",
            Self::MaxParallel => "Max Parallel Downloads",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{
        KeyCode::{self},
        KeyEvent, KeyModifiers,
    };
    use mediadl_core::config::AppConfig;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn config_state_holds_the_given_config() {
        let mut config = AppConfig::default();
        config.set_default_retries("7").unwrap();
        let state = ConfigState::new(config);
        assert_eq!(state.config.get_default_retries(), 7);
    }

    #[test]
    fn editing_retries_updates_config() {
        let mut state = ConfigState::new(AppConfig::default());
        state.selected = ConfigField::Retries;

        state.begin_edit();
        assert!(state.is_editing());

        // buffer starts pre-filled with the current value ("3") — clear it first
        for _ in 0..state.edit_text().len() {
            state.handle_edit_key(key(KeyCode::Backspace)).unwrap();
        }
        for c in "5".chars() {
            state.handle_edit_key(key(KeyCode::Char(c))).unwrap();
        }
        let result = state.handle_edit_key(key(KeyCode::Enter)).unwrap();

        assert!(!state.is_editing());
        assert_eq!(state.config.get_default_retries(), 5);
        assert_eq!(result, Some("Updated Retries".to_string()));
    }

    #[test]
    fn invalid_value_keeps_editing() {
        let mut state = ConfigState::new(AppConfig::default());
        state.selected = ConfigField::Retries;
        state.begin_edit();

        for _ in 0..state.edit_text().len() {
            state.handle_edit_key(key(KeyCode::Backspace)).unwrap();
        }
        for c in "abc".chars() {
            state.handle_edit_key(key(KeyCode::Char(c))).unwrap();
        }
        let result = state.handle_edit_key(key(KeyCode::Enter)).unwrap();

        assert!(state.is_editing()); // stayed open so they can fix it
        assert!(result.unwrap().starts_with("Error"));
    }
}
