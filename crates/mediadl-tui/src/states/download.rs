use crate::states::input::{EntryType, InputField, InputMode, TextInput};
use crate::states::output::OutputState;
use crate::traits::{Cycle, PanelNavigation, VerticalNavigation};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mediadl_core::download::{AudioDownloadRequest, VideoDownloadRequest, load_batch_file};

#[derive(Debug, Default)]
pub struct DownloadState {
    mode: DownloadType,
    focus: DownloadFocus,
    active_field: InputField,

    input_mode: InputMode,
    inputs: DownloadInputs,
}

#[derive(Debug, Default)]
pub enum DownloadFocus {
    #[default]
    Menu,
    Input,
    Output,
}

#[derive(Debug, Default, PartialEq)]
pub enum DownloadType {
    #[default]
    Video,
    Audio,
    Batch,
}

pub enum SubmitOutcome {
    Handled,
    StartVideo(VideoDownloadRequest),
    StartAudio(AudioDownloadRequest),
    StartVideoBatch(Vec<VideoDownloadRequest>),
    StartAudioBatch(Vec<AudioDownloadRequest>),
}

#[derive(Debug, Default)]
struct DownloadInputs {
    creator: TextInput,
    collection: TextInput,
    url: TextInput,
    kind: TextInput,
}

// download here refers to when the user is in the download screen
impl DownloadState {
    pub fn get_mode(&self) -> &DownloadType {
        &self.mode
    }

    pub fn is_editing(&self) -> bool {
        self.input_mode == InputMode::Edit
    }

    pub fn get_focus(&self) -> &DownloadFocus {
        &self.focus
    }

    pub fn clear_inputs(&mut self) {
        self.inputs.clear();
    }
    pub fn is_menu_focus(&self) -> bool {
        matches!(self.focus, DownloadFocus::Menu)
    }

    pub fn is_input_focus(&self) -> bool {
        matches!(self.focus, DownloadFocus::Input)
    }
    pub fn is_output_focus(&self) -> bool {
        matches!(self.focus, DownloadFocus::Output)
    }

    pub fn begin_edit(&mut self) {
        if matches!(self.focus, DownloadFocus::Input) {
            self.input_mode = InputMode::Edit;
        }
    }

    pub fn exit_edit(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn active_text(&self) -> &str {
        self.inputs.get(&self.active_field).text()
    }

    pub fn active_cursor(&self) -> usize {
        self.inputs.get(&self.active_field).cursor_position()
    }

    // cursor movements
    pub fn handle_edit_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if matches!(key_event.code, KeyCode::Esc) {
            self.exit_edit();
            return Ok(());
        }
        let input = self.inputs.get_mut(&self.active_field);

        match (key_event.code, key_event.modifiers) {
            (KeyCode::Backspace, mods) if mods.contains(KeyModifiers::CONTROL) => {
                input.delete_word()
            }
            (KeyCode::Char('w'), mods) if mods.contains(KeyModifiers::CONTROL) => {
                input.delete_word()
            }
            (KeyCode::Char(c), _) => input.insert_char(c),
            (KeyCode::Backspace, _) => input.delete_char(),
            (KeyCode::Left, _) => input.move_left(),
            (KeyCode::Right, _) => input.move_right(),
            (KeyCode::Home, _) => input.move_home(),
            (KeyCode::End, _) => input.move_end(),
            _ => {}
        }
        Ok(())
    }

    pub fn can_submit(&self) -> bool {
        // can only download when you're in the input panel and you're not editing
        matches!(self.focus, DownloadFocus::Input) && !self.is_editing()
    }

    pub fn submit(&mut self, output: &mut OutputState) -> SubmitOutcome {
        match self.mode {
            DownloadType::Video => self.submit_video(output),
            DownloadType::Audio => self.submit_audio(output),
            DownloadType::Batch => self.submit_batch(output),
        }
    }

    // for the ui
    pub fn field_items(&self) -> Vec<(&'static str, &str, bool)> {
        match self.mode {
            DownloadType::Batch => vec![
                (
                    "Batch File",
                    self.inputs.url.text(),
                    self.active_field == InputField::Url,
                ),
                (
                    "Type",
                    self.inputs.kind.text(),
                    self.active_field == InputField::Type,
                ),
            ],
            _ => vec![
                (
                    "Creator",
                    self.inputs.creator.text(),
                    self.active_field == InputField::Creator,
                ),
                (
                    "Collection",
                    self.inputs.collection.text(),
                    self.active_field == InputField::Collection,
                ),
                (
                    "Url",
                    self.inputs.url.text(),
                    self.active_field == InputField::Url,
                ),
            ],
        }
    }

    // selecting input fields -> helper function
    fn sync_active_field(&mut self) {
        self.active_field = match self.mode {
            DownloadType::Batch => InputField::Url,
            _ => InputField::Creator,
        };
    }

    // selecting input fields
    fn cycle_active_field(&mut self, forward: bool) {
        let fields: &[InputField] = match self.mode {
            DownloadType::Batch => &[InputField::Url, InputField::Type],
            _ => &[InputField::Creator, InputField::Collection, InputField::Url],
        };

        // navigating via indices
        let current = fields
            .iter()
            .position(|field| *field == self.active_field)
            .unwrap_or(0);

        let next = if forward {
            (current + 1) % fields.len()
        } else {
            (current + fields.len() - 1) % fields.len()
        };

        self.active_field = fields[next];
    }

    fn submit_video(&self, output: &mut OutputState) -> SubmitOutcome {
        let url = self.inputs.url.text().trim();

        if url.is_empty() {
            output.push_status("Error: Insert a link!".to_string());
            return SubmitOutcome::Handled;
        }

        let mut request = VideoDownloadRequest::new(url);
        let creator = self.inputs.creator.text().trim();

        if !creator.is_empty() {
            request.creator = Some(creator.to_string());
        }
        let collection = self.inputs.collection.text().trim();

        if !collection.is_empty() {
            request.collection = Some(collection.to_string());
        }

        output.push_status(format!("Starting video download: {url}"));
        SubmitOutcome::StartVideo(request)
    }
    fn submit_audio(&self, output: &mut OutputState) -> SubmitOutcome {
        let url = self.inputs.url.text().trim();

        if url.is_empty() {
            output.push_status("Insert a link!".to_string());
            return SubmitOutcome::Handled;
        }

        let mut request = AudioDownloadRequest::new(url);
        let creator = self.inputs.creator.text().trim();

        if !creator.is_empty() {
            request.creator = Some(creator.to_string());
        }

        let collection = self.inputs.collection.text().trim();

        if !collection.is_empty() {
            request.collection = Some(collection.to_string());
        }

        output.push_status(format!("Starting audio download: {url}"));
        SubmitOutcome::StartAudio(request)
    }

    fn submit_batch(&self, output: &mut OutputState) -> SubmitOutcome {
        let path = self.inputs.url.text().trim();

        if path.is_empty() {
            output.push_status("Error: Path cannot be empty".to_string());
            return SubmitOutcome::Handled;
        }

        let entry_type = match EntryType::parse(self.inputs.kind.text().trim()) {
            Ok(kind) => kind,
            Err(err) => {
                output.push_status(format!("Error: {err}"));
                return SubmitOutcome::Handled;
            }
        };

        let entries = match load_batch_file(path) {
            Ok(entries) => entries,
            Err(err) => {
                output.push_status(format!("Error: {err}"));
                return SubmitOutcome::Handled;
            }
        };

        if entries.is_empty() {
            output.push_status("Batch file has no downloads".to_string());
            return SubmitOutcome::Handled;
        }

        match entry_type {
            EntryType::Video => {
                // reads the file line by line
                let requests: Vec<VideoDownloadRequest> =
                    entries.into_iter().map(Into::into).collect();
                output.push_status(format!("Starting video batch ({} entries)", requests.len()));
                SubmitOutcome::StartVideoBatch(requests)
            }
            EntryType::Audio => {
                // reads the file line by line
                let requests: Vec<AudioDownloadRequest> =
                    entries.into_iter().map(Into::into).collect();
                output.push_status(format!("Starting audio batch ({} entries)", requests.len()));
                SubmitOutcome::StartAudioBatch(requests)
            }
        }
    }
}

impl DownloadInputs {
    pub fn clear(&mut self) {
        self.creator.set_text("");
        self.collection.set_text("");
        self.url.set_text("");
        self.kind.set_text("");
    }
    fn get(&self, field: &InputField) -> &TextInput {
        match field {
            InputField::Creator => &self.creator,
            InputField::Collection => &self.collection,
            InputField::Url => &self.url,
            InputField::Type => &self.kind,
        }
    }
    fn get_mut(&mut self, field: &InputField) -> &mut TextInput {
        match field {
            InputField::Creator => &mut self.creator,
            InputField::Collection => &mut self.collection,
            InputField::Url => &mut self.url,
            InputField::Type => &mut self.kind,
        }
    }
}

impl VerticalNavigation for DownloadState {
    fn move_up(&mut self) {
        match self.focus {
            DownloadFocus::Menu => {
                self.mode.prev();
                self.sync_active_field();
                self.clear_inputs();
            }
            DownloadFocus::Input => self.cycle_active_field(false),
            DownloadFocus::Output => {}
        }
    }
    fn move_down(&mut self) {
        match self.focus {
            DownloadFocus::Menu => {
                self.mode.next();
                self.sync_active_field();
                self.clear_inputs();
            }
            DownloadFocus::Input => self.cycle_active_field(true),
            DownloadFocus::Output => {}
        }
    }
}

impl PanelNavigation for DownloadState {
    fn forward(&mut self) {
        self.focus.next();
    }

    fn backward(&mut self) {
        self.focus.prev();
    }
}

impl Cycle for DownloadType {
    fn next(&mut self) {
        *self = match self {
            Self::Video => Self::Audio,
            Self::Audio => Self::Batch,
            Self::Batch => Self::Video,
        };
    }

    fn prev(&mut self) {
        *self = match self {
            Self::Video => Self::Batch,
            Self::Audio => Self::Video,
            Self::Batch => Self::Audio,
        };
    }
}

// impl Named for DownloadType {
//     fn name(&self) -> &'static str {
//         match self {
//             Self::Video => "Video",
//             Self::Audio => "Audio",
//             Self::Batch => "Batch",
//         }
//     }
// }

impl Cycle for DownloadFocus {
    fn next(&mut self) {
        *self = match self {
            Self::Menu => Self::Input,
            Self::Input => Self::Output,
            Self::Output => Self::Menu,
        }
    }
    fn prev(&mut self) {
        *self = match self {
            Self::Input => Self::Menu,
            Self::Output => Self::Menu,
            Self::Menu => Self::Output,
        }
    }
}

// impl Named for DownloadFocus {
//     fn name(&self) -> &'static str {
//         match self {
//             Self::Menu => "Menu",
//             Self::Input => "Input",
//             Self::Output => "Output",
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    fn to_batch(state: &mut DownloadState) {
        state.mode.next(); // Video -> Audio
        state.mode.next(); // Audio -> Batch
        state.sync_active_field();
    }

    #[test]
    fn batch_mode_shows_url_and_type_fields() {
        let mut state = DownloadState::default();
        to_batch(&mut state);
        let items = state.field_items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, "Batch File");
        assert_eq!(items[1].0, "Type");
    }

    #[test]
    fn test_audio_download_reject_empty_url() {
        let mut download_state = DownloadState::default();
        let mut output_state = OutputState::default();
        download_state.mode = DownloadType::Audio;

        let outcome = download_state.submit(&mut output_state);
        assert_eq!(output_state.lines()[0], "URL cannot be empty");
        assert!(matches!(outcome, SubmitOutcome::Handled));
    }

    #[test]
    fn test_video_download_reject_empty_url() {
        let mut download_state = DownloadState::default();
        let mut output_state = OutputState::default();
        download_state.mode = DownloadType::Video;
        let outcome = download_state.submit(&mut output_state);
        assert_eq!(output_state.lines()[0], "URL cannot be empty");
        assert!(matches!(outcome, SubmitOutcome::Handled));
    }

    // #[test]
    // fn test_download_video_request() {
    //     let mut download_state = DownloadState::default();
    //     let mut output_state = OutputState::default();
    //     let config = AppConfig::default();
    //
    //     download_state.mode = DownloadType::Video;
    //     download_state.submit(&mut output_state, &config);
    //
    //     assert_eq!(output_state.lines()[0], "Video download requested");
    // }
    //
    // #[test]
    // fn test_download_audio_request() {
    //     let mut download_state = DownloadState::default();
    //     let mut output_state = OutputState::default();
    //     let config = AppConfig::default();
    //
    //     download_state.mode = DownloadType::Audio;
    //     download_state.submit(&mut output_state, &config);
    //
    //     assert_eq!(output_state.lines()[0], "Audio download requested");
    // }
    //
    // #[test]
    // fn test_download_batch_request() {
    //     let mut download_state = DownloadState::default();
    //     let mut output_state = OutputState::default();
    //     let config = AppConfig::default();
    //
    //     download_state.mode = DownloadType::Batch;
    //     download_state.submit(&mut output_state, &config);
    //
    //     assert_eq!(output_state.lines()[0], "Batch download requested");
    // }
}

// we no longer call submit_* directly
// use mediadl_core::download::{
//     AudioDownloadRequest, VideoDownloadRequest, download_audio, download_audio_batch_parallel,
//     download_video, download_video_batch_parallel, load_batch_file,
// };
