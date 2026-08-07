pub mod config;
pub mod download;
pub mod input;
pub mod output;

#[derive(Debug, Default)]
pub enum Screen {
    #[default]
    Download,
    Config,
}

// DUMP: rubbish

// #[derive(Debug)]
// struct QueuedEntry {
//     url: String,
//     kind: EntryType,
// }
//
//
// #[test]
// fn enqueueing_valid_entry_adds_to_queue_and_clears_inputs() {
//     let mut state = DownloadState::default();
//     to_batch(&mut state);
//     state.inputs.url.set_text("example.com/video");
//     state.inputs.kind.set_text("video");
//
//     let mut output = OutputState::default();
//     state.submit(&mut output);
//
//     assert_eq!(state.queue_len(), 1);
//     assert_eq!(state.queued_entries()[0].url, "example.com/video");
//     assert!(matches!(state.queued_entries()[0].kind, EntryType::Video));
//     assert!(state.inputs.url.text().is_empty());
// }

// #[test]
// fn enqueueing_invalid_kind_reports_error_and_keeps_queue_empty() {
//     let mut state = DownloadState::default();
//     to_batch(&mut state);
//     state.inputs.url.set_text("example.com/video");
//     state.inputs.kind.set_text("nonsense");
//
//     let mut output = OutputState::default();
//     state.submit(&mut output);
//
//     assert_eq!(state.queue_len(), 0);
//     assert!(output.lines()[0].starts_with("Error"));
// }
// #[test]
// fn cycling_in_batch_mode_only_visits_url_and_kind() {
//     let mut state = DownloadState::default();
//     to_batch(&mut state);
//     state.focus = DownloadFocus::Input;
//     assert_eq!(state.active_field, InputField::Url);
//     state.cycle_active_field(true);
//     assert_eq!(state.active_field, InputField::Type);
//     state.cycle_active_field(true);
//     assert_eq!(state.active_field, InputField::Url);
// }
// #[test]
// fn submitting_empty_url_with_pending_queue_reports_ready() {
//     let mut state = DownloadState::default();
//     state.mode.next();
//     state.mode.next(); // -> Batch
//     state.sync_active_field();
//
//     state.inputs.url.set_text("example.com/a");
//     state.inputs.kind.set_text("audio");
//     let mut output = OutputState::default();
//     state.submit(&mut output); // enqueues, clears inputs
//
//     state.submit(&mut output); // url is empty again -> "ready" branch
//     assert_eq!(state.queue_len(), 1); // untouched — step 4 will drain it
//     assert!(output.lines().last().unwrap().contains("queued download"));
// }
//
//
// TODO: future work, it's like saving the things u want to download
// It's basically what the current batch function already does, but in app
//
// pub fn queue_len(&self) -> usize {
//     self.batch_queue.len()
// }

// fn _queued_entries(&self) -> &[QueuedEntry] {
//     &self.batch_queue
// }

// fn _take_batch_queue(&mut self) -> Vec<QueuedEntry> {
//     std::mem::take(&mut self.batch_queue)
// }

// fn enqueue_batch_entry(&mut self) -> Result<(), String> {
//     let url = self.inputs.url.text().trim();
//     if url.is_empty() {
//         return Err("URL cannot be empty".to_string());
//     }
//     let kind = EntryType::parse(self.inputs.kind.text())?;
//
//     self.batch_queue.push(QueuedEntry {
//         url: url.to_string(),
//         kind,
//     });
//     self.inputs.url.clear();
//     self.inputs.kind.clear();
//     Ok(())
// }
//
// TODO: future  for queuing downloads
// batch_queue: Vec<QueuedEntry>, <- inside DownloadState
