//! Configuration management

use directories::ProjectDirs;
use scarlett_core::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info};

/// Application preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    /// Enable keyboard volume control
    pub enable_hotkeys: bool,
    /// Volume step in dB for keyboard controls
    pub volume_step_db: f32,
    /// Last selected device serial number
    pub last_device_serial: Option<String>,
    /// Window positions and sizes
    pub window_geometry: WindowGeometry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub main_x: i32,
    pub main_y: i32,
    pub main_width: u32,
    pub main_height: u32,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            enable_hotkeys: true,
            volume_step_db: 1.0,
            last_device_serial: None,
            window_geometry: WindowGeometry {
                main_x: 100,
                main_y: 100,
                main_width: 800,
                main_height: 600,
            },
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "focusrite", "ScarlettGUI")
            .ok_or_else(|| Error::Config("Could not determine config directory".to_string()))?;

        let config_dir = project_dirs.config_dir().to_path_buf();

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
            info!("Created config directory: {:?}", config_dir);
        }

        Ok(Self { config_dir })
    }

    /// Load preferences
    pub fn load_preferences(&self) -> Result<Preferences> {
        let path = self.config_dir.join("preferences.ron");

        if !path.exists() {
            debug!("No preferences file found, using defaults");
            return Ok(Preferences::default());
        }

        let contents = std::fs::read_to_string(&path)?;
        let prefs = ron::from_str(&contents)
            .map_err(|e| Error::Config(format!("Failed to parse preferences: {}", e)))?;

        info!("Loaded preferences from {:?}", path);
        Ok(prefs)
    }

    /// Save preferences
    pub fn save_preferences(&self, prefs: &Preferences) -> Result<()> {
        let path = self.config_dir.join("preferences.ron");

        let contents = ron::ser::to_string_pretty(prefs, Default::default())
            .map_err(|e| Error::Config(format!("Failed to serialize preferences: {}", e)))?;

        std::fs::write(&path, contents)?;
        info!("Saved preferences to {:?}", path);
        Ok(())
    }

    /// Get device configuration path
    pub fn device_config_path(&self, serial: &str) -> PathBuf {
        self.config_dir.join(format!("device-{}.ron", serial))
    }

    /// Load device configuration
    pub fn load_device_config(&self, serial: &str) -> Result<DeviceConfig> {
        let path = self.device_config_path(serial);

        if !path.exists() {
            debug!("No device config found for {}, using defaults", serial);
            return Ok(DeviceConfig::default());
        }

        let contents = std::fs::read_to_string(&path)?;
        let config = ron::from_str(&contents)
            .map_err(|e| Error::Config(format!("Failed to parse device config: {}", e)))?;

        info!("Loaded device config for {} from {:?}", serial, path);
        Ok(config)
    }

    /// Save device configuration
    pub fn save_device_config(&self, serial: &str, config: &DeviceConfig) -> Result<()> {
        let path = self.device_config_path(serial);

        let contents = ron::ser::to_string_pretty(config, Default::default())
            .map_err(|e| Error::Config(format!("Failed to serialize device config: {}", e)))?;

        std::fs::write(&path, contents)?;
        info!("Saved device config for {} to {:?}", serial, path);
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create config manager")
    }
}

/// Device-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub routing: scarlett_core::routing::RoutingMatrix,
    pub mixer: scarlett_core::mixer::MixerState,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            routing: scarlett_core::routing::RoutingMatrix::new(),
            mixer: scarlett_core::mixer::MixerState::new(),
        }
    }
}
