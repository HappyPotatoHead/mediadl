// changes data
// generally anything that involves application logic goes here
use crate::event::{AppEvent, Event, EventHandler};
use crate::states::Screen;
use crate::states::config::ConfigState;
use crate::states::download::{DownloadFocus, DownloadState, SubmitOutcome};
use crate::states::output::OutputState;
use crate::traits::{PanelNavigation, VerticalNavigation};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mediadl_core::config::AppConfig;
use mediadl_core::download::{
    AudioDownloadRequest, VideoDownloadRequest, download_audio, download_audio_batch_parallel,
    download_video, download_video_batch_parallel,
};

use ratatui::DefaultTerminal;
use std::sync::Arc;

// application itself.
// define all the state that the application can be in
#[derive(Debug)]
pub struct App {
    // these two came with the template
    pub running: bool,
    pub events: EventHandler,

    // custom
    // input state should come in the future
    // for me to keep downloadstate to only handle download
    pub screen: Screen,
    pub download: DownloadState,
    pub output: OutputState,
    pub config: ConfigState,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        Self {
            running: true,
            events: EventHandler::new(),
            screen: Screen::default(),
            download: DownloadState::default(),
            output: OutputState::default(),
            config: ConfigState::new(config),
        }
    }

    // Run the application's main loop.
    // This came with the template
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        self.handle_key_events(key_event)?
                    }
                    _ => {}
                },
                // this is where the app's behaviour is connected to the app's event
                // anything new from AppEvent has to be included here
                Event::App(app_event) => match app_event {
                    AppEvent::MoveUp => self.move_up(),
                    AppEvent::MoveDown => self.move_down(),

                    AppEvent::Forward => self.forward(),
                    AppEvent::Backward => self.backward(),

                    AppEvent::Download => self.download(),

                    AppEvent::Back => self.back(),

                    AppEvent::OpenConfig => self.open_config(),
                    AppEvent::ShowOptions => self.show_options(),

                    AppEvent::Quit => self.quit(),

                    AppEvent::DownloadProgress(line) => self.output.push_status(line),
                    AppEvent::DownloadFinished(Ok(())) => {
                        self.output.push_status("Download finished".to_string())
                    }
                    AppEvent::DownloadFinished(Err(err)) => {
                        self.output.push_status(format!("Download failed: {err}"))
                    }
                },
            }
        }
        Ok(())
    }

    // handles the key events and updates the state of app
    // i split it into two types because it was getting too long
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match self.screen {
            Screen::Download => {
                if self.download.is_editing() {
                    self.download.handle_edit_key(key_event)?;
                } else {
                    self.handle_normal_key(key_event)?;
                }
            }
            Screen::Config => {
                if self.config.is_editing() {
                    if let Some(message) = self.config.handle_edit_key(key_event)? {
                        self.output.push_status(message);
                    }
                } else {
                    self.handle_normal_key(key_event)?;
                }
            }
        };

        Ok(())
    }

    fn handle_normal_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match (key_event.code, key_event.modifiers) {
            // reserved for only moving between panels
            // (KeyCode::Tab, mods) if mods.contains(KeyModifiers::SHIFT) => {
            //     self.events.send(AppEvent::Backward)
            // }
            (KeyCode::BackTab, _) => self.events.send(AppEvent::Backward),
            (KeyCode::Tab, _) => self.events.send(AppEvent::Forward),

            (KeyCode::Char('q'), mods) if mods.contains(KeyModifiers::CONTROL) => {
                self.events.send(AppEvent::Quit)
            }
            (KeyCode::Char('q'), _) => self.events.send(AppEvent::Back),

            // For the sake of convenience j/k should also be used to switch between
            // input fields
            // and also switching between fields in config screen, layout screen and colour screen
            // but layout screen and colour screen will have h and l in the future
            (KeyCode::Char('j'), _) => {
                let event = match self.download.get_focus() {
                    DownloadFocus::Menu => AppEvent::MoveDown,
                    DownloadFocus::Input => AppEvent::MoveUp,
                    DownloadFocus::Output => AppEvent::MoveDown,
                    // add in config in the future
                    // DownloadFocus::Config => AppEvent::MoveDown,
                };
                self.events.send(event);
            }

            (KeyCode::Char('k'), _) => {
                // add config focus in the future
                let event = match self.download.get_focus() {
                    DownloadFocus::Menu => AppEvent::MoveUp,
                    DownloadFocus::Input => AppEvent::MoveDown,
                    DownloadFocus::Output => AppEvent::MoveUp,
                    // add in config in the future
                    // DownloadFocus::Config => AppEvent::MoveDown,
                };
                self.events.send(event);
            }

            (KeyCode::Char('c'), _)
                if self.download.is_input_focus() && !self.download.is_editing() =>
            {
                self.download.clear_inputs();
            }
            (KeyCode::Char('c'), _) if self.download.is_output_focus() => {
                self.output.clear();
            }
            // This should be a global character
            (KeyCode::Char('C'), _) => self.events.send(AppEvent::ShowOptions),

            // this will only work if user is in Options menu
            (KeyCode::Char('e'), _) if self.output.is_options() => {
                self.events.send(AppEvent::OpenConfig)
            }

            // only works when inside input panel
            // this one means that pressing i anywhere will cause it to be edit mode
            (KeyCode::Char('i'), _) => match self.screen {
                Screen::Download => self.download.begin_edit(),
                Screen::Config => self.config.begin_edit(),
            },

            // for the sake of minimising user error, this can only occur when
            // selecting input panel AND in normal mode
            (KeyCode::Enter, _) if self.download.can_submit() && !self.output.is_options() => {
                self.events.send(AppEvent::Download);
            }

            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    fn move_up(&mut self) {
        match self.screen {
            Screen::Download => {
                match self.download.get_focus() {
                    DownloadFocus::Menu => self.download.move_up(),
                    DownloadFocus::Input => self.download.move_up(),
                    DownloadFocus::Output => self.output.scroll_up(),
                    // add in config in the future
                    // DownloadFocus::Config => AppEvent::MoveDown,
                };
            }
            Screen::Config => self.config.move_up(),
        }
    }

    fn move_down(&mut self) {
        match self.screen {
            Screen::Download => {
                match self.download.get_focus() {
                    DownloadFocus::Menu => self.download.move_down(),
                    DownloadFocus::Input => self.download.move_down(),
                    DownloadFocus::Output => self.output.scroll_down(),
                };
            }
            Screen::Config => self.config.move_down(),
        }
    }

    fn forward(&mut self) {
        match self.screen {
            Screen::Download => self.download.forward(),
            Screen::Config => {}
        }
    }
    fn backward(&mut self) {
        match self.screen {
            Screen::Download => self.download.backward(),
            Screen::Config => {}
        }
    }

    fn back(&mut self) {
        match self.screen {
            Screen::Download => {
                if self.output.is_options() {
                    self.output.show_status();
                }
            }
            // update this in the future
            Screen::Config => match self.config.save() {
                Ok(()) => {
                    self.output.push_status("Configuration saved".to_string());
                    self.screen = Screen::Download;
                }
                Err(err) => {
                    self.output
                        .push_status(format!("Failed to save configuration: {err}"));
                }
            },
        }
    }

    fn download(&mut self) {
        if !matches!(self.screen, Screen::Download) {
            return;
        }
        match self.download.submit(&mut self.output) {
            SubmitOutcome::Handled => {}
            SubmitOutcome::StartVideo(request) => self.spawn_video(request),
            SubmitOutcome::StartAudio(request) => self.spawn_audio(request),
            SubmitOutcome::StartVideoBatch(request) => self.spawn_video_batch(request),
            SubmitOutcome::StartAudioBatch(request) => self.spawn_audio_batch(request),
        }
    }

    fn spawn_video(&mut self, request: VideoDownloadRequest) {
        let finished_sender = self.events.sender();
        let progress_sender = finished_sender.clone();
        let config = Arc::clone(&self.config.config);

        let request = request.with_on_line(move |line| {
            let _ = progress_sender.send(Event::App(AppEvent::DownloadProgress(line)));
        });
        tokio::task::spawn_blocking(move || {
            let result = download_video(request, &config);
            let _ = finished_sender.send(Event::App(AppEvent::DownloadFinished(result)));
        });
    }
    fn spawn_audio(&mut self, request: AudioDownloadRequest) {
        let finished_sender = self.events.sender();
        let progress_sender = finished_sender.clone();
        let config = Arc::clone(&self.config.config);
        let request = request.with_on_line(move |line| {
            let _ = progress_sender.send(Event::App(AppEvent::DownloadProgress(line)));
        });
        tokio::task::spawn_blocking(move || {
            let result = download_audio(request, &config);
            let _ = finished_sender.send(Event::App(AppEvent::DownloadFinished(result)));
        });
    }

    fn spawn_video_batch(&mut self, requests: Vec<VideoDownloadRequest>) {
        let finished_sender = self.events.sender();
        let progress_sender = finished_sender.clone();
        let config = Arc::clone(&self.config.config);
        let requests: Vec<VideoDownloadRequest> = requests
            .into_iter()
            .map(|req| {
                let sender = progress_sender.clone();
                req.with_on_line(move |line| {
                    let _ = sender.send(Event::App(AppEvent::DownloadProgress(line)));
                })
            })
            .collect();
        tokio::task::spawn_blocking(move || {
            let result = download_video_batch_parallel(&requests, &config);
            let _ = finished_sender.send(Event::App(AppEvent::DownloadFinished(result)));
        });
    }

    fn spawn_audio_batch(&mut self, requests: Vec<AudioDownloadRequest>) {
        let finished_sender = self.events.sender();
        let progress_sender = finished_sender.clone();
        let config = Arc::clone(&self.config.config);
        let requests: Vec<AudioDownloadRequest> = requests
            .into_iter()
            .map(|req| {
                let sender = progress_sender.clone();
                req.with_on_line(move |line| {
                    let _ = sender.send(Event::App(AppEvent::DownloadProgress(line)));
                })
            })
            .collect();
        tokio::task::spawn_blocking(move || {
            let result = download_audio_batch_parallel(&requests, &config);
            let _ = finished_sender.send(Event::App(AppEvent::DownloadFinished(result)));
        });
    }

    fn open_config(&mut self) {
        if matches!(self.screen, Screen::Download) && self.output.is_options() {
            self.output.show_status();
            self.screen = Screen::Config;
        }
    }

    fn show_options(&mut self) {
        if matches!(self.screen, Screen::Download) {
            self.output.show_options();
        }
    }
}

// app.rs
#[cfg(test)]
mod tests {
    use super::*;
    use mediadl_core::config::AppConfig;

    // when it comes to tokio, use #[tokio::test]
    #[tokio::test]
    async fn new_app_starts_running_on_download_screen() {
        let app = App::new(AppConfig::default());
        assert!(app.running);
        assert!(matches!(app.screen, Screen::Download));
    }

    #[tokio::test]
    async fn c_shows_the_options_submenu() {
        let mut app = App::new(AppConfig::default());
        app.show_options();
        assert!(app.output.is_options());
    }

    #[tokio::test]
    async fn e_only_enters_config_from_the_options_view() {
        let mut app = App::new(AppConfig::default());
        app.open_config();
        assert!(matches!(app.screen, Screen::Download));

        app.show_options();
        app.open_config();
        assert!(matches!(app.screen, Screen::Config));
        assert!(!app.output.is_options());
    }

    #[tokio::test]
    async fn q_from_options_view_returns_to_status() {
        let mut app = App::new(AppConfig::default());
        app.show_options();
        app.back();
        assert!(!app.output.is_options());
        assert!(matches!(app.screen, Screen::Download));
    }
}
