// changes data
use crate::event::{AppEvent, Event, EventHandler};
use crate::states::Screen;
use crate::states::config::ConfigState;
use crate::states::download::DownloadState;
use crate::states::output::OutputState;
use crate::traits::{PanelNavigation, VerticalNavigation};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mediadl_core::config::AppConfig;

use ratatui::DefaultTerminal;

// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub screen: Screen,
    pub download: DownloadState,
    pub output: OutputState,
    pub config: ConfigState,
}

impl App {
    /// Constructs a new instance of [`App`].
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

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
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
                Event::App(app_event) => match app_event {
                    // AppEvent::Increment => self.increment_counter(),
                    // AppEvent::Decrement => self.decrement_counter(),
                    // AppEvent::DownloadMode => self.change_download_mode(),
                    AppEvent::MoveUp => self.move_up(),
                    AppEvent::MoveDown => self.move_down(),

                    AppEvent::Forward => self.forward(),
                    AppEvent::Backward => self.backward(),

                    AppEvent::Download => self.download(),

                    AppEvent::Back => self.back(),

                    AppEvent::OpenConfig => self.open_config(),
                    AppEvent::ShowOptions => self.show_options(),
                    AppEvent::Quit => self.quit(),
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
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
            Screen::Download => self.download.move_up(),
            Screen::Config => self.config.move_up(),
        }
    }

    fn move_down(&mut self) {
        match self.screen {
            Screen::Download => self.download.move_down(),
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
        match self.screen {
            Screen::Download => self.download.submit(&mut self.output, &self.config.config),
            Screen::Config => {}
        }
    }

    fn open_config(&mut self) {
        if matches!(self.screen, Screen::Download) && self.output.is_options() {
            self.output.show_status();
            self.screen = Screen::Config;
        }
    }

    fn handle_normal_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match (key_event.code, key_event.modifiers) {
            // reserved for only moving between panels
            (KeyCode::Tab, mods) if mods.contains(KeyModifiers::SHIFT) => {
                self.events.send(AppEvent::Backward)
            }
            (KeyCode::Tab, _) => self.events.send(AppEvent::Forward),

            (KeyCode::Char('q'), mods) if mods.contains(KeyModifiers::CONTROL) => {
                self.events.send(AppEvent::Quit)
            }
            (KeyCode::Char('q'), _) => self.events.send(AppEvent::Back),

            // For the sake of convenience j/k should also be used to switch between
            // input fields
            // and also switching between fields in config screen, layout screen and colour screen
            // but layout screen and colour screen will have h and l in the future
            (KeyCode::Char('j'), _) => self.events.send(AppEvent::MoveDown),
            (KeyCode::Char('k'), _) => self.events.send(AppEvent::MoveUp),

            // This should be a global character
            (KeyCode::Char('C'), _) => self.events.send(AppEvent::ShowOptions),
            (KeyCode::Char('e'), _) => self.events.send(AppEvent::OpenConfig),

            // only works when inside input panel
            // this one means that pressing i anywhere will cause it to be edit mode
            // (KeyCode::Char('i'), _) => self.download.edit(),
            (KeyCode::Char('i'), _) => match self.screen {
                Screen::Download => self.download.begin_edit(),
                Screen::Config => self.config.begin_edit(),
            },

            // (KeyCode::Esc, _) => self.events.send(AppEvent::ExitEdit),

            // For the sake of minimising user error, this can only occur when
            // selecting input panel AND in normal mode
            (KeyCode::Enter, _) if self.download.can_submit() && !self.output.is_options() => {
                self.events.send(AppEvent::Download);
            }

            _ => {}
        }
        Ok(())
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
