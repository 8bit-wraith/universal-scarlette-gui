//! Mixer data structures

use serde::{Deserialize, Serialize};

/// Mixer channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerChannel {
    /// Channel index
    pub index: usize,
    /// Channel name
    pub name: String,
    /// Volume in dB (-127.0 to +6.0 typical)
    pub volume_db: f32,
    /// Pan value (-1.0 = left, 0.0 = center, 1.0 = right)
    pub pan: f32,
    /// Mute state
    pub muted: bool,
    /// Solo state
    pub solo: bool,
    /// Is this channel part of a stereo pair?
    pub stereo_pair: Option<usize>,
}

impl MixerChannel {
    pub fn new(index: usize, name: String) -> Self {
        Self {
            index,
            name,
            volume_db: 0.0,
            pan: 0.0,
            muted: false,
            solo: false,
            stereo_pair: None,
        }
    }

    /// Convert dB to linear gain (0.0 to 1.0+)
    pub fn volume_linear(&self) -> f32 {
        db_to_linear(self.volume_db)
    }

    /// Set volume from linear gain
    pub fn set_volume_linear(&mut self, gain: f32) {
        self.volume_db = linear_to_db(gain);
    }
}

/// Mixer state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerState {
    /// All mixer channels
    pub channels: Vec<MixerChannel>,
    /// Master volume in dB
    pub master_volume_db: f32,
    /// Master mute
    pub master_muted: bool,
}

impl MixerState {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            master_volume_db: 0.0,
            master_muted: false,
        }
    }
}

impl Default for MixerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Level meter data
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LevelMeter {
    /// Current level in dB (-127.0 to 0.0)
    pub level_db: f32,
    /// Peak level in dB
    pub peak_db: f32,
}

impl LevelMeter {
    pub fn new() -> Self {
        Self {
            level_db: -127.0,
            peak_db: -127.0,
        }
    }

    /// Update level and peak
    pub fn update(&mut self, new_level_db: f32) {
        self.level_db = new_level_db;
        if new_level_db > self.peak_db {
            self.peak_db = new_level_db;
        }
    }

    /// Reset peak
    pub fn reset_peak(&mut self) {
        self.peak_db = self.level_db;
    }
}

impl Default for LevelMeter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert dB to linear gain
pub fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Convert linear gain to dB
pub fn linear_to_db(gain: f32) -> f32 {
    if gain > 0.0 {
        20.0 * gain.log10()
    } else {
        -127.0 // Minimum dB
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_conversion() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 0.001);
        assert!((db_to_linear(-6.0) - 0.501).abs() < 0.001);
        assert!((db_to_linear(6.0) - 1.995).abs() < 0.01);
    }

    #[test]
    fn test_linear_conversion() {
        assert!((linear_to_db(1.0) - 0.0).abs() < 0.001);
        assert!((linear_to_db(0.5) - (-6.02)).abs() < 0.01);
    }
}
