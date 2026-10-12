//! Responsive UI implementation inspired by the VLC Community Redesign (Figma 1691465999134370980).
//! Features floating pill dock controls, timeline scrubber with dual timestamps,
//! volume popover, hardware acceleration indicator, and slide-out playlist drawer.

use crate::player::{HwDecMode, PlaybackState};
use crate::theme::{Palette, ThemeMode};
use iced::widget::{
    button, column, container, horizontal_space, row, slider, text, text_input, vertical_space,
};
use iced::{Alignment, Color, Command, Element, Length};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub id: String,
    pub title: String,
    pub artist_or_source: String,
    pub duration_seconds: f64,
    pub path_or_url: String,
    pub codec_tag: String,
}

#[derive(Debug, Clone)]
pub enum UiMessage {
    TogglePlayPause,
    SeekTo(f64),
    SeekRelative(f64),
    VolumeChanged(f64),
    ToggleMute,
    ToggleTheme,
    TogglePlaylistDrawer,
    SelectHwDec(HwDecMode),
    SelectPlaylistItem(String),
    AddMediaPrompt,
    SearchPlaylistChanged(String),
    ToggleFullscreen,
    CycleSpeed,
    CycleSubtitles,
    CycleAudio,
    Tick,
}

pub struct SuperPlayerUi {
    pub theme_mode: ThemeMode,
    pub is_playlist_open: bool,
    pub search_query: String,
    pub playlist: Vec<PlaylistItem>,
    pub active_track_id: Option<String>,
    pub is_fullscreen: bool,
}

impl Default for SuperPlayerUi {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::Dark,
            is_playlist_open: false,
            search_query: String::new(),
            playlist: vec![
                PlaylistItem {
                    id: "1".into(),
                    title: "Cosmos 4K UltraHD Demo (60fps)".into(),
                    artist_or_source: "HEVC / HDR10 Direct Stream".into(),
                    duration_seconds: 245.0,
                    path_or_url: "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4".into(),
                    codec_tag: "HEVC / NVDEC".into(),
                },
                PlaylistItem {
                    id: "2".into(),
                    title: "Tears of Steel (Open Source VFX)".into(),
                    artist_or_source: "Blender Studio / 1080p".into(),
                    duration_seconds: 734.0,
                    path_or_url: "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/TearsOfSteel.mp4".into(),
                    codec_tag: "H.264 / VAAPI".into(),
                },
                PlaylistItem {
                    id: "3".into(),
                    title: "Sintel - The Durian Project".into(),
                    artist_or_source: "Surround 5.1 / Opus".into(),
                    duration_seconds: 888.0,
                    path_or_url: "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/Sintel.mp4".into(),
                    codec_tag: "AV1 / MediaCodec".into(),
                },
            ],
            active_track_id: Some("1".into()),
            is_fullscreen: false,
        }
    }
}

// Custom container styling implementation for iced 0.12
struct CustomBoxStyle {
    bg: Color,
    border_color: Color,
    border_width: f32,
    radius: f32,
    shadow: bool,
}

impl container::StyleSheet for CustomBoxStyle {
    type Style = iced::Theme;
    fn appearance(&self, _theme: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(self.bg.into()),
            border: iced::Border {
                color: self.border_color,
                width: self.border_width,
                radius: self.radius.into(),
            },
            shadow: if self.shadow {
                iced::Shadow {
                    color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.35 },
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 16.0,
                }
            } else {
                iced::Shadow::default()
            },
            ..Default::default()
        }
    }
}

fn box_style(bg: Color, border_color: Color, border_width: f32, radius: f32, shadow: bool) -> iced::theme::Container {
    iced::theme::Container::Custom(Box::new(CustomBoxStyle {
        bg,
        border_color,
        border_width,
        radius,
        shadow,
    }))
}

impl SuperPlayerUi {
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the main window layout
    pub fn view<'a>(&'a self, state: &'a PlaybackState) -> Element<'a, UiMessage> {
        let palette = Palette::from_mode(self.theme_mode);

        // 1. Top Bar: App Title, Current Hardware Decoder, Light/Dark toggle, Playlist Toggle
        let top_bar = self.render_top_bar(&palette, state);

        // 2. Video Viewport Stage
        let video_viewport = self.render_video_viewport(&palette, state);

        // 3. Floating Player Controls Dock & Timeline Scrubber
        let floating_dock = self.render_floating_dock(&palette, state);

        // Combine video area + floating dock overlay
        let center_content = column![
            video_viewport,
            floating_dock,
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);

        // 4. Main Body: Center content + optional Playlist Drawer
        let main_body = if self.is_playlist_open {
            row![
                center_content,
                self.render_playlist_drawer(&palette, state),
            ]
            .spacing(16)
            .width(Length::Fill)
            .height(Length::Fill)
        } else {
            row![center_content]
                .width(Length::Fill)
                .height(Length::Fill)
        };

        let full_layout = column![top_bar, main_body]
            .spacing(12)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill);

        container(full_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(box_style(palette.background, Color::TRANSPARENT, 0.0, 0.0, false))
            .into()
    }

    /// Top Bar adhering to the clean VLC Redesign aesthetics
    fn render_top_bar<'a>(&'a self, palette: &Palette, state: &'a PlaybackState) -> Element<'a, UiMessage> {
        let theme_label = match self.theme_mode {
            ThemeMode::Dark => "☀ Light",
            ThemeMode::Light => "🌙 Dark",
        };

        let hwdec_badge = if state.is_hwdec_active {
            format!("⚡ HW ACCEL: {}", state.hwdec_current.to_uppercase())
        } else {
            "⚡ HW: AUTO (NVDEC/VAAPI/MediaCodec)".to_string()
        };

        row![
            // Brand Logo & Title
            row![
                text("SUPER PLAYER").size(15),
                text("·").size(14),
                text(&state.media_title).size(13),
            ]
            .spacing(8)
            .align_items(Alignment::Center),

            horizontal_space(),

            // Hardware Decoder Status Indicator
            container(text(hwdec_badge).size(11))
                .padding([4, 10])
                .style(box_style(palette.accent_subtle, palette.accent, 1.0, 6.0, false)),

            // Theme Toggle
            button(text(theme_label).size(12))
                .on_press(UiMessage::ToggleTheme)
                .padding([6, 12]),

            // Playlist Drawer Toggle
            button(
                text(if self.is_playlist_open { "Hide Queue" } else { "Queue (3)" }).size(12)
            )
            .on_press(UiMessage::TogglePlaylistDrawer)
            .padding([6, 12]),
        ]
        .spacing(12)
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .into()
    }

    /// Video Viewport placeholder or active native canvas
    fn render_video_viewport<'a>(&'a self, palette: &Palette, state: &'a PlaybackState) -> Element<'a, UiMessage> {
        let hud_overlay = column![
            vertical_space(),
            row![
                text(format!("Codec: {} | Audio: {}", state.video_codec, state.audio_codec))
                    .size(11),
                horizontal_space(),
                text(format!("Speed: {:.2}x", state.speed)).size(11),
            ]
            .padding(16)
            .width(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        container(hud_overlay)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(box_style(palette.surface, palette.border_subtle, 1.0, 12.0, false))
            .into()
    }

    /// Floating Player Controls dock inspired by Figma VLC Redesign
    fn render_floating_dock<'a>(&'a self, palette: &Palette, state: &'a PlaybackState) -> Element<'a, UiMessage> {
        let current_time_str = format_time(state.time_pos);
        let total_time_str = format_time(state.duration);

        // Timeline Scrubber
        let timeline_row = row![
            text(current_time_str).size(12),
            slider(0.0..=100.0, state.percent_pos, UiMessage::SeekTo)
                .step(0.1)
                .width(Length::Fill),
            text(total_time_str).size(12),
        ]
        .spacing(12)
        .align_items(Alignment::Center)
        .width(Length::Fill);

        // Control Buttons Row
        let play_icon = if state.is_playing { "⏸" } else { "▶" };
        let volume_icon = if state.is_muted || state.volume == 0.0 { "🔇" } else { "🔊" };

        let controls_row = row![
            // Left Group: Subtitles & Audio
            button(text("Subtitles").size(12)).on_press(UiMessage::CycleSubtitles),
            button(text("Audio").size(12)).on_press(UiMessage::CycleAudio),

            horizontal_space(),

            // Center Floating Dock: Prev, -10s, Play/Pause, +10s, Next
            button(text("⏮").size(14)).on_press(UiMessage::SeekRelative(-30.0)),
            button(text("-10s").size(12)).on_press(UiMessage::SeekRelative(-10.0)),
            button(text(play_icon).size(18))
                .on_press(UiMessage::TogglePlayPause)
                .padding([8, 20]),
            button(text("+10s").size(12)).on_press(UiMessage::SeekRelative(10.0)),
            button(text("⏭").size(14)).on_press(UiMessage::SeekRelative(30.0)),

            horizontal_space(),

            // Right Group: Speed, Volume, Fullscreen
            button(text(format!("{:.1}x", state.speed)).size(12)).on_press(UiMessage::CycleSpeed),
            button(text(volume_icon).size(14)).on_press(UiMessage::ToggleMute),
            slider(0.0..=150.0, state.volume, UiMessage::VolumeChanged).width(Length::Fixed(80.0)),
            button(text("⛶").size(14)).on_press(UiMessage::ToggleFullscreen),
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .width(Length::Fill);

        let dock_container = column![timeline_row, controls_row]
            .spacing(12)
            .padding([14, 20]);

        container(dock_container)
            .width(Length::Fill)
            .style(box_style(palette.surface_floating, palette.border, 1.0, 16.0, true))
            .into()
    }

    /// Playlist Drawer
    fn render_playlist_drawer<'a>(&'a self, palette: &Palette, _state: &'a PlaybackState) -> Element<'a, UiMessage> {
        let search_bar = text_input("Filter tracks...", &self.search_query)
            .on_input(UiMessage::SearchPlaylistChanged)
            .padding(8);

        let mut tracks_col = column![].spacing(8);

        for item in &self.playlist {
            if !self.search_query.is_empty()
                && !item.title.to_lowercase().contains(&self.search_query.to_lowercase())
            {
                continue;
            }

            let is_selected = self.active_track_id.as_deref() == Some(&item.id);
            let item_id = item.id.clone();

            let item_row = row![
                column![
                    text(&item.title).size(13),
                    text(&item.artist_or_source).size(11),
                ]
                .spacing(2),
                horizontal_space(),
                text(format_time(item.duration_seconds)).size(11),
            ]
            .align_items(Alignment::Center)
            .padding(8);

            let item_btn = button(item_row)
                .on_press(UiMessage::SelectPlaylistItem(item_id))
                .width(Length::Fill);

            tracks_col = tracks_col.push(item_btn);
        }

        let drawer_content = column![
            row![
                text("Queue & Playlist").size(14),
                horizontal_space(),
                button(text("+ Add Media").size(11)).on_press(UiMessage::AddMediaPrompt),
            ]
            .align_items(Alignment::Center),
            search_bar,
            tracks_col,
        ]
        .spacing(12)
        .padding(16);

        container(drawer_content)
            .width(Length::Fixed(320.0))
            .height(Length::Fill)
            .style(box_style(palette.surface, palette.border, 1.0, 12.0, false))
            .into()
    }
}

/// Helper to format seconds to MM:SS or HH:MM:SS
fn format_time(seconds: f64) -> String {
    let total_secs = seconds.max(0.0) as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}
