//! Audio module — ambient loops and one-shot UI sounds via Bevy's built-in audio.
//!
//! Uses `bevy_audio` (with vorbis support) for spatial and non-spatial
//! audio playback, driven by message-based commands.

mod events;
mod manager;
mod plugin;

pub use events::{AudioCommand, UiSound};
pub use manager::AudioManager;
pub use plugin::DeerAudioPlugin;
