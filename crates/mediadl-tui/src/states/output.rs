use std::cell::Cell;
#[derive(Debug, Default)]
pub struct OutputState {
    status: Vec<String>,
    view: OutputView,
    scroll_offset: usize,
    follow_tail: bool,
    last_max_scroll: Cell<usize>,
}

#[derive(Debug, Default)]
enum OutputView {
    #[default]
    Status,
    Options,
}

impl OutputState {
    pub fn clear(&mut self) {
        // clears everything
        self.status.clear();
        // reset scroll to top
        self.scroll_offset = 0;
    }
    pub fn lines(&self) -> &[String] {
        &self.status
    }
    pub fn push_status(&mut self, message: impl Into<String>) {
        let msg = message.into();

        for raw_part in msg.split(['\n', '\r']) {
            let part = raw_part.trim().to_string();
            if part.is_empty() {
                continue;
            }

            if part.starts_with("[info]")
                || part.starts_with("[ThumbnailsConvertor]")
                || part.starts_with("[youtube:tab]")
                || part.contains("Destination:")
                || part.contains("mutagen:")
                || part.contains("Downloading android vr player")
                || part.contains("Downloading webpage")
                || part.contains("pass -k to keep")
                || part.contains("Extracting URL:")
                || part.contains("Deleting existing file")
            {
                continue;
            }

            let is_progress = part.contains("[download]") && part.contains("%");
            if is_progress
                && let Some(last) = self.status.last_mut()
                && last.starts_with("[download]")
                && last.contains("%")
            {
                *last = part;
                self.follow_tail = true;
                continue;
            }
            self.status.push(part);
            self.follow_tail = true;
        }
    }

    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }
    pub fn set_last_max_scroll(&self, max_scroll: usize) {
        self.last_max_scroll.set(max_scroll);
    }

    pub fn scroll_up(&mut self) {
        let max_scroll = self.last_max_scroll.get();
        if self.follow_tail {
            self.follow_tail = false;
            self.scroll_offset = max_scroll.saturating_sub(1);
        } else {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        }
    }

    pub fn scroll_down(&mut self) {
        let max_scroll = self.last_max_scroll.get();
        let next = self.scroll_offset.saturating_add(1);

        if next >= max_scroll {
            self.follow_tail = true;
            self.scroll_offset = max_scroll;
        } else {
            self.scroll_offset = next;
        }
    }

    pub fn follow_tail(&self) -> bool {
        self.follow_tail
    }

    pub fn is_options(&self) -> bool {
        matches!(self.view, OutputView::Options)
    }

    pub fn show_status(&mut self) {
        self.view = OutputView::Status;
    }

    pub fn toggle_view(&mut self) {
        self.view = match self.view {
            OutputView::Status => OutputView::Options,
            OutputView::Options => OutputView::Status,
        }
    }
}

// impl Cycle for OptionsSelections{
//     fn next(&mut self){
//         *self
//     }
//     }
