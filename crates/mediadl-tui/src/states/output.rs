#[derive(Debug, Default)]
pub struct OutputState {
    status: Vec<String>,
    view: OutputView,
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
        self.status.push(message.into());
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
