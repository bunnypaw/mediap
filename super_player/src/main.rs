//! Super Player - High-performance native media player in pure Rust.
//! Linked natively to libmpv (C API) with hardware acceleration.
//! Designed according to the VLC Community Redesign (Figma 1691465999134370980).

mod player;
mod theme;
mod ui;

use anyhow::Result;
use iced::executor;
use iced::time;
use iced::window;
use iced::{Application, Command, Element, Length, Settings, Subscription};
use player::{HwDecMode, MpvPlayer, PlaybackState};
use std::time::Duration;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use ui::{SuperPlayerUi, UiMessage};

pub fn main() -> iced::Result {
    // Initialize logging filter
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting Super Player (Rust + libmpv native engine)");

    let settings = Settings {
        window: window::Settings {
            size: iced::Size::new(1200.0, 780.0),
            min_size: Some(iced::Size::new(800.0, 520.0)),
            decorations: true,
            transparent: false,
            ..Default::default()
        },
        flags: (),
        default_font: iced::Font::DEFAULT,
        default_text_size: iced::Pixels(14.0),
        antialiasing: true,
    };

    SuperPlayerApp::run(settings)
}

struct SuperPlayerApp {
    player: Option<MpvPlayer>,
    ui: SuperPlayerUi,
    cached_state: PlaybackState,
}

#[derive(Debug, Clone)]
enum Message {
    Ui(UiMessage),
    FileOpened(Option<String>),
    StateRefreshed(PlaybackState),
}

impl Application for SuperPlayerApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Initialize the native libmpv engine with Hardware Acceleration Auto mode
        let player = match MpvPlayer::new(HwDecMode::Auto) {
            Ok(p) => {
                info!("libmpv engine initialized successfully with NVDEC/VAAPI/MediaCodec detection");
                Some(p)
            }
            Err(err) => {
                error!("Warning: libmpv initialization failed: {}. Running in preview mode.", err);
                None
            }
        };

        let ui = SuperPlayerUi::new();
        let cached_state = PlaybackState::default();

        (
            Self {
                player,
                ui,
                cached_state,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("Super Player - {}", self.cached_state.media_title)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Ui(ui_msg) => match ui_msg {
                UiMessage::TogglePlayPause => {
                    if let Some(ref player) = self.player {
                        let _ = player.toggle_pause();
                    }
                    self.cached_state.is_playing = !self.cached_state.is_playing;
                    self.cached_state.is_paused = !self.cached_state.is_playing;
                    Command::none()
                }
                UiMessage::SeekTo(percent) => {
                    if let Some(ref player) = self.player {
                        let target_sec = (percent / 100.0) * self.cached_state.duration;
                        let _ = player.seek_absolute(target_sec);
                    }
                    self.cached_state.percent_pos = percent;
                    Command::none()
                }
                UiMessage::SeekRelative(seconds) => {
                    if let Some(ref player) = self.player {
                        let _ = player.seek_relative(seconds);
                    }
                    self.cached_state.time_pos = (self.cached_state.time_pos + seconds).max(0.0);
                    Command::none()
                }
                UiMessage::VolumeChanged(vol) => {
                    if let Some(ref player) = self.player {
                        let _ = player.set_volume(vol);
                    }
                    self.cached_state.volume = vol;
                    Command::none()
                }
                UiMessage::ToggleMute => {
                    if let Some(ref player) = self.player {
                        if let Ok(next) = player.toggle_mute() {
                            self.cached_state.is_muted = next;
                        }
                    } else {
                        self.cached_state.is_muted = !self.cached_state.is_muted;
                    }
                    Command::none()
                }
                UiMessage::ToggleTheme => {
                    self.ui.theme_mode = self.ui.theme_mode.toggle();
                    Command::none()
                }
                UiMessage::TogglePlaylistDrawer => {
                    self.ui.is_playlist_open = !self.ui.is_playlist_open;
                    Command::none()
                }
                UiMessage::SelectHwDec(mode) => {
                    if let Some(ref mut player) = self.player {
                        let _ = player.set_hwdec(mode);
                    }
                    Command::none()
                }
                UiMessage::SelectPlaylistItem(id) => {
                    if let Some(item) = self.ui.playlist.iter().find(|i| i.id == id).cloned() {
                        self.ui.active_track_id = Some(id);
                        self.cached_state.media_title = item.title;
                        self.cached_state.duration = item.duration_seconds;
                        if let Some(ref player) = self.player {
                            let _ = player.load_file(&item.path_or_url);
                        }
                    }
                    Command::none()
                }
                UiMessage::AddMediaPrompt => {
                    // Open native file dialog using rfd
                    Command::perform(
                        async {
                            let file = rfd::AsyncFileDialog::new()
                                .add_filter("Video & Audio", &["mp4", "mkv", "avi", "webm", "flv", "mp3", "flac", "wav", "m4a", "m3u8"])
                                .set_title("Open Media File")
                                .pick_file()
                                .await;

                            file.map(|f| f.path().to_string_lossy().to_string())
                        },
                        Message::FileOpened,
                    )
                }
                UiMessage::SearchPlaylistChanged(query) => {
                    self.ui.search_query = query;
                    Command::none()
                }
                UiMessage::ToggleFullscreen => {
                    self.ui.is_fullscreen = !self.ui.is_fullscreen;
                    let mode = if self.ui.is_fullscreen {
                        window::Mode::Fullscreen
                    } else {
                        window::Mode::Windowed
                    };
                    window::set_mode(window::Id::MAIN, mode)
                }
                UiMessage::CycleSpeed => {
                    let next_speed = match self.cached_state.speed {
                        s if s < 0.9 => 1.0,
                        s if s < 1.2 => 1.25,
                        s if s < 1.4 => 1.5,
                        s if s < 1.8 => 2.0,
                        _ => 0.5,
                    };
                    self.cached_state.speed = next_speed;
                    if let Some(ref player) = self.player {
                        let _ = player.set_speed(next_speed);
                    }
                    Command::none()
                }
                UiMessage::CycleSubtitles => {
                    if let Some(ref player) = self.player {
                        let _ = player.cycle_subtitles();
                    }
                    Command::none()
                }
                UiMessage::CycleAudio => {
                    if let Some(ref player) = self.player {
                        let _ = player.cycle_audio();
                    }
                    Command::none()
                }
                UiMessage::Tick => {
                    if let Some(ref player) = self.player {
                        self.cached_state = player.get_state();
                    }
                    Command::none()
                }
            },
            Message::FileOpened(Some(path)) => {
                info!("Selected file: {}", path);
                let title = std::path::Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Custom File")
                    .to_string();

                let id = format!("{}", self.ui.playlist.len() + 1);
                self.ui.playlist.push(ui::PlaylistItem {
                    id: id.clone(),
                    title: title.clone(),
                    artist_or_source: "Local Media".into(),
                    duration_seconds: 0.0,
                    path_or_url: path.clone(),
                    codec_tag: "Native".into(),
                });
                self.ui.active_track_id = Some(id);
                self.cached_state.media_title = title;
                if let Some(ref player) = self.player {
                    let _ = player.load_file(&path);
                }
                Command::none()
            }
            Message::FileOpened(None) => Command::none(),
            Message::StateRefreshed(state) => {
                self.cached_state = state;
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        // Poll mpv engine telemetry every 250ms for smooth timeline rendering
        time::every(Duration::from_millis(250)).map(|_| Message::Ui(UiMessage::Tick))
    }

    fn view(&self) -> Element<Message> {
        self.ui.view(&self.cached_state).map(Message::Ui)
    }
}
