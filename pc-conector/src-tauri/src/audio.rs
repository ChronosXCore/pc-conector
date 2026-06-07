use crate::network::AudioStreamConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, Sample, Stream, StreamConfig};
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use std::thread;

use serde::{Serialize, Deserialize};

/// Información de un dispositivo de audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub device_type: AudioDeviceType,
    pub is_default: bool,
    pub channels: u16,
    pub sample_rates: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioDeviceType {
    Input,
    Output,
    Both,
}

/// Servicio de streaming de audio bidireccional
pub struct AudioService {
    host: cpal::Host,
    is_streaming: Arc<Mutex<bool>>,
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    input_device: Option<String>,
    output_device: Option<String>,
    on_audio_data: Option<Box<dyn Fn(Vec<f32>) + Send + 'static>>,
    sample_rate: u32,
}

unsafe impl Send for AudioService {}
unsafe impl Sync for AudioService {}

impl AudioService {
    pub fn new() -> Self {
        let host = cpal::default_host();
        
        Self {
            host,
            is_streaming: Arc::new(Mutex::new(false)),
            input_stream: None,
            output_stream: None,
            input_device: None,
            output_device: None,
            on_audio_data: None,
            sample_rate: 48000,
        }
    }

    /// Obtener lista de dispositivos de audio disponibles
    pub fn list_devices(&self) -> Result<(Vec<AudioDeviceInfo>, Vec<AudioDeviceInfo>), String> {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        let default_input = self.host.default_input_device();
        let default_output = self.host.default_output_device();

        for device in self.host.devices().map_err(|e| format!("Error al listar dispositivos: {}", e))? {
            let name = device.name().unwrap_or_default();
            let is_default_input = default_input.as_ref().map(|d| d.name().ok()) == Some(Some(name.clone()));
            let is_default_output = default_output.as_ref().map(|d| d.name().ok()) == Some(Some(name.clone()));

            // Verificar si es dispositivo de entrada
            if let Ok(config) = device.default_input_config() {
                inputs.push(AudioDeviceInfo {
                    name: name.clone(),
                    device_type: AudioDeviceType::Input,
                    is_default: is_default_input,
                    channels: config.channels(),
                    sample_rates: vec![config.sample_rate().0],
                });
            }

            // Verificar si es dispositivo de salida
            if let Ok(config) = device.default_output_config() {
                outputs.push(AudioDeviceInfo {
                    name,
                    device_type: AudioDeviceType::Output,
                    is_default: is_default_output,
                    channels: config.channels(),
                    sample_rates: vec![config.sample_rate().0],
                });
            }
        }

        Ok((inputs, outputs))
    }

    /// Establecer callback para datos de audio recibidos
    pub fn set_on_audio_data<F>(&mut self, callback: F)
    where
        F: Fn(Vec<f32>) + Send + 'static,
    {
        self.on_audio_data = Some(Box::new(callback));
    }

    /// Iniciar captura de audio desde el micrófono
    pub fn start_capture(&mut self, device_name: Option<&str>) -> Result<(), String> {
        let device = self.select_device(device_name, true)?;
        let config = device.default_input_config()
            .map_err(|e| format!("Error al obtener configuración de entrada: {}", e))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let is_streaming = self.is_streaming.clone();

        *is_streaming.lock().unwrap() = true;

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if *is_streaming.lock().unwrap() {
                    // Aquí se enviarían los datos de audio al peer
                    // Por ahora solo lo registramos
                    info!("Audio capturado: {} samples", data.len());
                }
            },
            |err| {
                error!("Error en stream de entrada: {}", err);
            },
            None,
        ).map_err(|e| format!("Error al crear stream de entrada: {}", e))?;

        stream.play().map_err(|e| format!("Error al iniciar stream: {}", e))?;
        self.input_stream = Some(stream);
        info!("Captura de audio iniciada ({} Hz, {} canales)", sample_rate, channels);
        Ok(())
    }

    /// Iniciar reproducción de audio (para recibir audio del peer)
    pub fn start_playback(&mut self, device_name: Option<&str>) -> Result<(), String> {
        let device = self.select_device(device_name, false)?;
        let config = device.default_output_config()
            .map_err(|e| format!("Error al obtener configuración de salida: {}", e))?;

        let channels = config.channels() as usize;

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Aquí se recibirían datos del peer para reproducir
                // Por ahora silencio
                for sample in data.iter_mut() {
                    *sample = 0.0;
                }
            },
            |err| {
                error!("Error en stream de salida: {}", err);
            },
            None,
        ).map_err(|e| format!("Error al crear stream de salida: {}", e))?;

        stream.play().map_err(|e| format!("Error al iniciar playback: {}", e))?;
        self.output_stream = Some(stream);
        info!("Reproducción de audio iniciada ({} canales)", channels);
        Ok(())
    }

    /// Seleccionar un dispositivo por nombre
    fn select_device(&self, name: Option<&str>, is_input: bool) -> Result<Device, String> {
        if let Some(device_name) = name {
            for device in self.host.devices().map_err(|e| format!("Error: {}", e))? {
                if device.name().map(|n| n == device_name).unwrap_or(false) {
                    return Ok(device);
                }
            }
            warn!("Dispositivo '{}' no encontrado, usando default", device_name);
        }

        if is_input {
            self.host.default_input_device()
                .ok_or_else(|| "No hay dispositivo de entrada predeterminado".to_string())
        } else {
            self.host.default_output_device()
                .ok_or_else(|| "No hay dispositivo de salida predeterminado".to_string())
        }
    }

    /// Detener todo el streaming de audio
    pub fn stop(&mut self) {
        *self.is_streaming.lock().unwrap() = false;
        self.input_stream = None;
        self.output_stream = None;
        info!("Audio detenido");
    }
}
