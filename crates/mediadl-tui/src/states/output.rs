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
    pub fn lines(&self) -> &[String] {
        &self.status
    }
    pub fn push_status(&mut self, message: impl Into<String>) {
        let msg = message.into();

        if msg.contains("Downloading android vr player")
            || msg.contains("Downloading webpage")
            || msg.contains("pass -k to keep")
            || msg.contains("Extracting URL:")
        {
            return;
        }

        let is_progress = msg.contains("[download]") && msg.contains("%");
        if is_progress {
            if let Some(last) = self.status.last_mut() {
                if last.starts_with("[download]") && last.contains("%") {
                    *last = msg;
                    self.follow_tail = true;
                    return;
                }
            }
        }
        self.status.push(msg);
        self.follow_tail = true;
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
        // self.scroll_offset = self.scroll_offset.saturating_add(1);
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

        // self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
    pub fn follow_tail(&self) -> bool {
        self.follow_tail
    }

    pub fn is_options(&self) -> bool {
        matches!(self.view, OutputView::Options)
    }
    pub fn show_options(&mut self) {
        self.view = OutputView::Options;
    }

    pub fn show_status(&mut self) {
        self.view = OutputView::Status;
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn push_status_resets_scroll_to_latest() {
//         let mut state = OutputState::default();
//         for i in 0..5 {
//             state.push_status(format!("line {i}"));
//         }
//         state.scroll_up();
//         state.scroll_up();
//         assert_eq!(state.scroll_offset(), 2);
//
//         state.push_status("new line".to_string());
//         assert_eq!(state.scroll_offset(), 0);
//     }
//
//     #[test]
//     fn scroll_up_clamps_to_available_lines() {
//         let mut state = OutputState::default();
//         state.push_status("only line".to_string());
//         state.scroll_up();
//         state.scroll_up();
//         assert_eq!(state.scroll_offset(), 0); // nothing before the only line
//     }
//
//     #[test]
//     fn scroll_down_does_not_go_below_zero() {
//         let mut state = OutputState::default();
//         state.push_status("a".to_string());
//         state.scroll_down();
//         assert_eq!(state.scroll_offset(), 0);
//     }
// }
