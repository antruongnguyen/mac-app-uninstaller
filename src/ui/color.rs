//! Centralized color definitions for the UI.
//!
//! This module groups all app colors for better maintainability and theming.
//! Prefer importing colors from here instead of hard-coding Color32 values.

use egui::Color32;

/// Semantic color roles independent of specific RGB values.
/// These are grouped to make it easier to theme the app.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    // Surfaces
    pub window_bg: Color32,
    pub panel_bg: Color32,

    // Content text
    pub text: Color32,
    pub text_muted: Color32,
    pub text_inverse: Color32,

    // Accents and interactive states
    pub accent: Color32,
    pub hover_bg: Color32,
    pub border: Color32,

    // Status / feedback
    pub warning: Color32,
    pub danger: Color32,
    pub info: Color32,
    pub success: Color32,

    // List specific helpers
    pub list_bg_default: Color32,
    pub list_bg_hover: Color32,
    pub list_bg_selected: Color32,
}

impl Palette {
    /// A macOS/AppKit-like light palette.
    pub const fn light() -> Self {
        Self {
            window_bg: Color32::from_rgb(252, 250, 244),
            panel_bg: Color32::from_rgb(247, 248, 250),

            text: Color32::from_rgb(30, 30, 30),
            text_muted: Color32::from_rgb(110, 112, 124),
            text_inverse: Color32::WHITE,

            accent: Color32::from_rgb(58, 128, 246),
            hover_bg: Color32::from_rgb(245, 245, 247),
            border: Color32::from_rgb(235, 236, 240),

            warning: Color32::from_rgb(200, 140, 30),
            danger: Color32::from_rgb(212, 96, 104),
            info: Color32::from_rgb(60, 120, 220),
            success: Color32::from_rgb(82, 148, 87),

            list_bg_default: Color32::from_rgb(247, 248, 250),
            list_bg_hover: Color32::WHITE,
            list_bg_selected: Color32::from_rgb(58, 128, 246),
        }
    }
}

/// Returns the active palette. Currently always light, but can be extended to
/// read user settings or system theme.
pub fn palette() -> Palette {
    Palette::light()
}

/// Convenience aliases for frequently used roles.
pub mod roles {
    use super::{palette, Color32};

    pub fn window_bg() -> Color32 {
        palette().window_bg
    }
    pub fn panel_bg() -> Color32 {
        palette().panel_bg
    }

    pub fn text() -> Color32 {
        palette().text
    }
    pub fn text_muted() -> Color32 {
        palette().text_muted
    }
    pub fn text_inverse() -> Color32 {
        palette().text_inverse
    }

    pub fn accent() -> Color32 {
        palette().accent
    }
    pub fn hover_bg() -> Color32 {
        palette().hover_bg
    }
    pub fn border() -> Color32 {
        palette().border
    }

    pub fn warning() -> Color32 {
        palette().warning
    }
    pub fn danger() -> Color32 {
        palette().danger
    }
    pub fn info() -> Color32 {
        palette().info
    }
    pub fn success() -> Color32 {
        palette().success
    }

    pub fn list_bg_default() -> Color32 {
        palette().list_bg_default
    }
    pub fn list_bg_hover() -> Color32 {
        palette().list_bg_hover
    }
    pub fn list_bg_selected() -> Color32 {
        palette().list_bg_selected
    }
}
