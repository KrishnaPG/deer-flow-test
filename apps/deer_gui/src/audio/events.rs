//! Audio command messages — drive the audio system from other modules.
//!
//! Other systems send [`AudioCommand`] messages to request playback,
//! volume changes, and mute/unmute without coupling to audio internals.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Audio command message
// ---------------------------------------------------------------------------

/// Commands sent to the audio system via the message bus.
#[derive(Message, Debug, Clone)]
pub enum AudioCommand {
    /// Start playing an ambient loop track with a fade-in.
    PlayAmbient {
        /// Asset path of the audio file.
        track: String,
        /// Fade-in duration in seconds.
        fade_in_secs: f32,
    },
    /// Stop the current ambient loop with a fade-out.
    StopAmbient {
        /// Fade-out duration in seconds.
        fade_out_secs: f32,
    },
    /// Play a one-shot UI sound effect.
    PlayOneShot {
        /// Which UI sound to play.
        sound: UiSound,
    },
    /// Set the master volume (0.0–1.0).
    SetMasterVolume {
        /// New volume level.
        volume: f32,
    },
    /// Mute all audio output.
    Mute,
    /// Unmute audio output (restores previous volume).
    Unmute,
}

// ---------------------------------------------------------------------------
// UI sound identifiers
// ---------------------------------------------------------------------------

/// One-shot UI sound identifiers.
///
/// Each variant maps to an audio asset loaded on startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiSound {
    /// Generic button click.
    Click,
    /// Agent approval chime.
    ApprovalChime,
    /// Error notification tone.
    ErrorTone,
    /// Alert ping (incoming event).
    AlertPing,
    /// Camera movement whoosh.
    CameraMove,
    /// Mission completion fanfare.
    MissionComplete,
}

impl UiSound {
    /// Returns the asset path for this sound.
    ///
    /// Paths are relative to the `assets/` directory.
    pub fn asset_path(&self) -> &'static str {
        match self {
            Self::Click => "sounds/click.ogg",
            Self::ApprovalChime => "sounds/approval.ogg",
            Self::ErrorTone => "sounds/error.ogg",
            Self::AlertPing => "sounds/alert.ogg",
            Self::CameraMove => "sounds/camera_move.ogg",
            Self::MissionComplete => "sounds/mission_complete.ogg",
        }
    }
}
