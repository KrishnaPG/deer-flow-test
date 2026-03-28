//! Audio manager resource — tracks master volume, mute state, and active ambient track.

use bevy::log::{debug, trace};
use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Audio state resource managing volume and ambient track info.
#[derive(Resource, Debug)]
pub struct AudioManager {
    /// Master volume (0.0–1.0).
    pub master_volume: f32,
    /// Whether audio is muted.
    pub muted: bool,
    /// Currently playing ambient track asset path (if any).
    pub current_ambient: Option<String>,
    /// Entity holding the ambient audio source (for stop/fade).
    pub ambient_entity: Option<Entity>,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioManager {
    /// Creates a new audio manager with default settings.
    pub fn new() -> Self {
        debug!("AudioManager::new — master_volume=1.0, muted=false");
        Self {
            master_volume: 1.0,
            muted: false,
            current_ambient: None,
            ambient_entity: None,
        }
    }

    /// Returns the effective volume (0.0 if muted).
    pub fn effective_volume(&self) -> f32 {
        if self.muted {
            trace!("AudioManager::effective_volume — muted, returning 0.0");
            0.0
        } else {
            trace!("AudioManager::effective_volume — {}", self.master_volume);
            self.master_volume
        }
    }

    /// Sets the master volume (clamped to 0.0–1.0).
    pub fn set_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        debug!(
            "AudioManager::set_volume — master_volume={}",
            self.master_volume
        );
    }

    /// Mutes audio output.
    pub fn mute(&mut self) {
        debug!("AudioManager::mute");
        self.muted = true;
    }

    /// Unmutes audio output.
    pub fn unmute(&mut self) {
        debug!("AudioManager::unmute");
        self.muted = false;
    }
}
