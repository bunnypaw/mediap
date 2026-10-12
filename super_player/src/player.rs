//! Safe Rust wrapper for libmpv (C API) with hardware acceleration.
//! Supports NVDEC (NVIDIA), VAAPI (Intel/AMD Linux), and MediaCodec (Android ARM64).

use anyhow::{Context, Result};
use libmpv2::{events::Event, FileState, Format, Mpv};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

/// Target hardware acceleration decoding backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HwDecMode {
    #[default]
    Auto,
    /// NVIDIA NVDEC via CUDA / Direct3D11
    Nvdec,
    /// Linux Intel/AMD Video Acceleration API
    Vaapi,
    /// Android / ARM64 NDK MediaCodec
    MediaCodec,
    /// Software decoding (CPU fallback)
    Off,
}

impl HwDecMode {
    pub fn as_mpv_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto-safe",
            Self::Nvdec => "nvdec-copy",
            Self::Vaapi => "vaapi-copy",
            Self::MediaCodec => "mediacodec",
            Self::Off => "no",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Auto => "AUTO (Hardware Safe)",
            Self::Nvdec => "NVDEC (NVIDIA)",
            Self::Vaapi => "VAAPI (Linux/Mesa)",
            Self::MediaCodec => "MediaCodec (Android ARM64)",
            Self::Off => "Software (CPU Only)",
        }
    }
}

/// Snapshot of the current player telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub is_paused: bool,
    pub is_idle: bool,
    pub time_pos: f64,
    pub duration: f64,
    pub percent_pos: f64,
    pub volume: f64,
    pub is_muted: bool,
    pub speed: f64,
    pub media_title: String,
    pub media_path: String,
    pub video_codec: String,
    pub audio_codec: String,
    pub hwdec_current: String,
    pub is_hwdec_active: bool,
    pub sub_tracks_count: usize,
    pub audio_tracks_count: usize,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playing: false,
            is_paused: true,
            is_idle: true,
            time_pos: 0.0,
            duration: 0.0,
            percent_pos: 0.0,
            volume: 100.0,
            is_muted: false,
            speed: 1.0,
            media_title: String::from("No Media Loaded"),
            media_path: String::new(),
            video_codec: String::from("None"),
            audio_codec: String::from("None"),
            hwdec_current: String::from("None"),
            is_hwdec_active: false,
            sub_tracks_count: 0,
            audio_tracks_count: 0,
        }
    }
}

/// Safe thread-aware wrapper managing the native libmpv instance
pub struct MpvPlayer {
    mpv: Arc<Mutex<Mpv>>,
    hwdec_mode: HwDecMode,
    is_shutting_down: Arc<AtomicBool>,
}

impl MpvPlayer {
    /// Initialize libmpv instance and configure hardware acceleration & GPU pipeline
    pub fn new(hwdec: HwDecMode) -> Result<Self> {
        info!("Initializing native libmpv context with hwdec={:?}", hwdec);

        let mut mpv = Mpv::new().context("Failed to allocate native mpv_handle")?;

        // 1. Hardware acceleration configuration
        mpv.set_property("hwdec", hwdec.as_mpv_str())
            .context("Failed to set hwdec property")?;

        // 2. High-performance rendering pipeline configuration
        // vo=gpu-next provides modern Vulkan/OpenGL/D3D11 rendering
        let _ = mpv.set_property("vo", "gpu-next");
        let _ = mpv.set_property("gpu-context", "auto");
        let _ = mpv.set_property("hwdec-codecs", "all");

        // 3. Audio & Playback resilience
        let _ = mpv.set_property("keep-open", "yes");
        let _ = mpv.set_property("hr-seek", "yes");
        let _ = mpv.set_property("force-seekable", "yes");
        let _ = mpv.set_property("ytdl", "yes");
        let _ = mpv.set_property("cache", "yes");
        let _ = mpv.set_property("demuxer-max-bytes", "150MiB");
        let _ = mpv.set_property("volume-max", 150.0);

        // 4. Observe properties for UI synchronization
        mpv.observe_property("time-pos", Format::Double, 1)?;
        mpv.observe_property("duration", Format::Double, 2)?;
        mpv.observe_property("pause", Format::Flag, 3)?;
        mpv.observe_property("media-title", Format::String, 4)?;
        mpv.observe_property("hwdec-current", Format::String, 5)?;
        mpv.observe_property("video-codec", Format::String, 6)?;

        let mpv_arc = Arc::new(Mutex::new(mpv));
        let is_shutting_down = Arc::new(AtomicBool::new(false));

        Ok(Self {
            mpv: mpv_arc,
            hwdec_mode: hwdec,
            is_shutting_down,
        })
    }

    /// Load a local file or network stream (HLS / DASH / MP4)
    pub fn load_file(&self, path_or_url: &str) -> Result<()> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        info!("Loading media into libmpv: {}", path_or_url);
        mpv.command(&["loadfile", path_or_url, "replace"])
            .context("Failed to execute mpv loadfile command")?;
        // Start playing immediately on load
        let _ = mpv.set_property("pause", false);
        Ok(())
    }

    /// Append a file to the playback queue
    pub fn append_to_playlist(&self, path_or_url: &str) -> Result<()> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        mpv.command(&["loadfile", path_or_url, "append"])
            .context("Failed to append file to playlist")?;
        Ok(())
    }

    /// Toggle play / pause state
    pub fn toggle_pause(&self) -> Result<bool> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        let is_paused: bool = mpv.get_property("pause").unwrap_or(false);
        let next_state = !is_paused;
        mpv.set_property("pause", next_state)?;
        Ok(next_state)
    }

    pub fn play(&self) -> Result<()> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        mpv.set_property("pause", false)?;
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        mpv.set_property("pause", true)?;
        Ok(())
    }

    /// Relative seek in seconds (+10s or -10s)
    pub fn seek_relative(&self, seconds: f64) -> Result<()> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        mpv.command(&["seek", &seconds.to_string(), "relative+exact"])?;
        Ok(())
    }

    /// Absolute seek to a specific second in the media
    pub fn seek_absolute(&self, seconds: f64) -> Result<()> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        mpv.command(&["seek", &seconds.to_string(), "absolute+exact"])?;
        Ok(())
    }

    /// Set volume (0.0 to 100.0, or up to 150.0 boost)
    pub fn set_volume(&self, volume: f64) -> Result<()> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        let clamped = volume.clamp(0.0, 150.0);
        mpv.set_property("volume", clamped)?;
        Ok(())
    }

    /// Toggle mute
    pub fn toggle_mute(&self) -> Result<bool> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        let is_muted: bool = mpv.get_property("mute").unwrap_or(false);
        let next = !is_muted;
        mpv.set_property("mute", next)?;
        Ok(next)
    }

    /// Change playback speed (e.g., 0.5x, 1.0x, 1.25x, 1.5x, 2.0x)
    pub fn set_speed(&self, speed: f64) -> Result<()> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        let clamped = speed.clamp(0.25, 4.0);
        mpv.set_property("speed", clamped)?;
        Ok(())
    }

    /// Switch hardware acceleration mode at runtime
    pub fn set_hwdec(&mut self, mode: HwDecMode) -> Result<()> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        self.hwdec_mode = mode;
        mpv.set_property("hwdec", mode.as_mpv_str())?;
        info!("Updated hardware decoder to: {}", mode.label());
        Ok(())
    }

    /// Cycle subtitle track
    pub fn cycle_subtitles(&self) -> Result<()> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        mpv.command(&["cycle", "sub"])?;
        Ok(())
    }

    /// Cycle audio track
    pub fn cycle_audio(&self) -> Result<()> {
        let mpv = self.mpv.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        mpv.command(&["cycle", "audio"])?;
        Ok(())
    }

    /// Fetch current snapshot of player telemetry
    pub fn get_state(&self) -> PlaybackState {
        let Ok(mpv) = self.mpv.lock() else {
            return PlaybackState::default();
        };

        let is_paused: bool = mpv.get_property("pause").unwrap_or(true);
        let is_idle: bool = mpv.get_property("idle-active").unwrap_or(true);
        let time_pos: f64 = mpv.get_property("time-pos").unwrap_or(0.0);
        let duration: f64 = mpv.get_property("duration").unwrap_or(0.0);
        let percent_pos: f64 = mpv.get_property("percent-pos").unwrap_or(0.0);
        let volume: f64 = mpv.get_property("volume").unwrap_or(100.0);
        let is_muted: bool = mpv.get_property("mute").unwrap_or(false);
        let speed: f64 = mpv.get_property("speed").unwrap_or(1.0);
        let media_title: String = mpv.get_property("media-title").unwrap_or_else(|_| String::from("No Media Loaded"));
        let media_path: String = mpv.get_property("path").unwrap_or_default();
        let video_codec: String = mpv.get_property("video-codec").unwrap_or_else(|_| String::from("None"));
        let audio_codec: String = mpv.get_property("audio-codec").unwrap_or_else(|_| String::from("None"));
        let hwdec_current: String = mpv.get_property("hwdec-current").unwrap_or_else(|_| String::from("no"));
        let is_hwdec_active = !hwdec_current.is_empty() && hwdec_current != "no";

        PlaybackState {
            is_playing: !is_paused && !is_idle,
            is_paused,
            is_idle,
            time_pos,
            duration,
            percent_pos,
            volume,
            is_muted,
            speed,
            media_title,
            media_path,
            video_codec,
            audio_codec,
            hwdec_current,
            is_hwdec_active,
            sub_tracks_count: 1,
            audio_tracks_count: 1,
        }
    }

    /// Graceful shutdown
    pub fn shutdown(&self) {
        self.is_shutting_down.store(true, Ordering::SeqCst);
        if let Ok(mpv) = self.mpv.lock() {
            let _ = mpv.command(&["quit"]);
        }
    }
}
