//! Theme system inspired by the VLC Community Redesign (Figma 1691465999134370980).
//! Supports seamless Light and Dark mode switching, translucent floating surfaces,
//! signature VLC orange accenting, and high-contrast typography.

use iced::{color, Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
}

impl ThemeMode {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

pub struct Palette {
    pub background: Color,
    pub surface: Color,
    pub surface_elevated: Color,
    pub surface_floating: Color,
    pub border: Color,
    pub border_subtle: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_subtle: Color,
    pub scrubber_track: Color,
    pub scrubber_buffer: Color,
    pub control_active: Color,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            background: color!(0x0E, 0x11, 0x17),
            surface: color!(0x16, 0x1B, 0x22),
            surface_elevated: color!(0x1F, 0x24, 0x2C),
            surface_floating: Color {
                r: 0.08,
                g: 0.10,
                b: 0.14,
                a: 0.88,
            },
            border: color!(0x30, 0x36, 0x3D),
            border_subtle: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.08,
            },
            text_primary: color!(0xF0, 0xF6, 0xFC),
            text_secondary: color!(0x8B, 0x94, 0x9E),
            text_muted: color!(0x6E, 0x76, 0x81),
            // Signature vibrant VLC orange
            accent: color!(0xFF, 0x77, 0x00),
            accent_hover: color!(0xFF, 0x8C, 0x24),
            accent_subtle: Color {
                r: 1.0,
                g: 0.47,
                b: 0.0,
                a: 0.15,
            },
            scrubber_track: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.16,
            },
            scrubber_buffer: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.32,
            },
            control_active: color!(0xFF, 0x77, 0x00),
        }
    }

    pub fn light() -> Self {
        Self {
            background: color!(0xF8, 0xF9, 0xFA),
            surface: color!(0xFF, 0xFF, 0xFF),
            surface_elevated: color!(0xF1, 0xF3, 0xF5),
            surface_floating: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.92,
            },
            border: color!(0xDE, 0xE2, 0xE6),
            border_subtle: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.06,
            },
            text_primary: color!(0x21, 0x25, 0x29),
            text_secondary: color!(0x49, 0x50, 0x57),
            text_muted: color!(0x86, 0x8E, 0x96),
            accent: color!(0xFA, 0x64, 0x00),
            accent_hover: color!(0xE0, 0x55, 0x00),
            accent_subtle: Color {
                r: 0.98,
                g: 0.39,
                b: 0.0,
                a: 0.12,
            },
            scrubber_track: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.12,
            },
            scrubber_buffer: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.24,
            },
            control_active: color!(0xFA, 0x64, 0x00),
        }
    }

    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        }
    }
}
