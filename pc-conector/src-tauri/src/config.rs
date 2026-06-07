use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuración completa de la aplicación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub services: ServicesConfig,
    pub screens: Vec<ScreenConfig>,
    pub audio: AudioConfig,
    pub connection: ConnectionConfig,
    #[serde(default)]
    pub linked_devices: Vec<LinkedDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub auto_start: bool,
    pub auto_connect: bool,
    pub language: String,
    pub theme: String,
    pub minimize_to_tray: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    pub clipboard_sync: bool,
    pub mouse_sharing: bool,
    pub keyboard_sharing: bool,
    pub audio_sharing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenConfig {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub stream_microphone: bool,
    pub stream_speakers: bool,
    pub sample_rate: u32,
    pub bitrate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub peer_address: Option<String>,
    pub auto_reconnect: bool,
    pub reconnect_interval: u64,
    pub encryption_enabled: bool,
    pub security_token: String,
}

/// A trusted device that has been "linked" for auto-connect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedDevice {
    pub ip: String,
    pub name: String,
    pub linked_at: u64, // Unix timestamp seconds
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                auto_start: false,
                auto_connect: false,
                language: "es".to_string(),
                theme: "dark".to_string(),
                minimize_to_tray: true,
            },
            services: ServicesConfig {
                clipboard_sync: true,
                mouse_sharing: true,
                keyboard_sharing: true,
                audio_sharing: false,
            },
            screens: Vec::new(),
            audio: AudioConfig {
                input_device: None,
                output_device: None,
                stream_microphone: false,
                stream_speakers: false,
                sample_rate: 48000,
                bitrate: 64000,
            },
            connection: ConnectionConfig {
                peer_address: None,
                auto_reconnect: true,
                reconnect_interval: 5,
                encryption_enabled: true,
                security_token: "123456".to_string(),
            },
            linked_devices: Vec::new(),
        }
    }
}

impl AppConfig {
    /// Ruta del archivo de configuración
    pub fn config_path() -> PathBuf {
        let mut path = dirs_next::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("pc-conector");
        path.push("config.json");
        path
    }

    /// Cargar configuración desde disco
    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
        Self::default()
    }

    /// Guardar configuración en disco
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Información de un peer descubierto en la red
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub hostname: String,
    pub ip_address: String,
    pub port: u16,
    pub os: String,
    pub version: String,
}
