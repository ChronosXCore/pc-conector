use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream};
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
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

/// Servicio de streaming de audio bidireccional con codificación Opus
pub struct AudioService {
    host: cpal::Host,
    is_streaming: Arc<Mutex<bool>>,
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    #[allow(dead_code)]
    input_device: Option<String>,
    #[allow(dead_code)]
    output_device: Option<String>,
    on_encoded_data: Arc<Mutex<Option<Box<dyn Fn(Vec<u8>) + Send + 'static>>>>,
    playback_buffer: Arc<Mutex<Vec<f32>>>,
    decoder: Arc<Mutex<Option<opus::Decoder>>>,
    output_channels: Arc<Mutex<u16>>,
    #[allow(dead_code)]
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
            on_encoded_data: Arc::new(Mutex::new(None)),
            playback_buffer: Arc::new(Mutex::new(Vec::new())),
            decoder: Arc::new(Mutex::new(None)),
            output_channels: Arc::new(Mutex::new(1)),
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

    /// Establecer callback para datos de audio codificados listos para enviar
    pub fn set_on_encoded_data<F>(&mut self, callback: F)
    where
        F: Fn(Vec<u8>) + Send + 'static,
    {
        *self.on_encoded_data.lock().unwrap() = Some(Box::new(callback));
    }

    /// Iniciar captura de audio desde el micrófono y codificarlo con Opus
    pub fn start_capture(&mut self, device_name: Option<&str>) -> Result<(), String> {
        let device = self.select_device(device_name, true)?;
        let config = device.default_input_config()
            .map_err(|e| format!("Error al obtener configuración de entrada: {}", e))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let is_streaming = self.is_streaming.clone();

        *is_streaming.lock().unwrap() = true;

        // Crear codificador Opus (48000Hz, Mono para streaming eficiente)
        let mut encoder = opus::Encoder::new(48000, opus::Channels::Mono, opus::Application::Audio)
            .map_err(|e| format!("Error al crear codificador Opus: {:?}", e))?;

        let capture_buffer = Arc::new(Mutex::new(Vec::new()));
        let on_encoded = self.on_encoded_data.clone();

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if *is_streaming.lock().unwrap() {
                    let mut buffer = capture_buffer.lock().unwrap();
                    buffer.extend_from_slice(data);

                    // Un frame de 20ms a 48000Hz Mono contiene exactamente 960 samples
                    while buffer.len() >= 960 {
                        let frame: Vec<f32> = buffer.drain(0..960).collect();
                        let mut output = vec![0u8; 1000];
                        match encoder.encode_float(&frame, &mut output) {
                            Ok(len) => {
                                output.truncate(len);
                                if let Some(ref cb) = *on_encoded.lock().unwrap() {
                                    cb(output);
                                }
                            }
                            Err(e) => {
                                error!("Error al codificar audio Opus: {:?}", e);
                            }
                        }
                    }
                }
            },
            |err| {
                error!("Error en stream de entrada: {}", err);
            },
            None,
        ).map_err(|e| format!("Error al crear stream de entrada: {}", e))?;

        stream.play().map_err(|e| format!("Error al iniciar stream: {}", e))?;
        self.input_stream = Some(stream);
        info!("Captura de audio iniciada ({} Hz, {} canales, codificación Opus Mono)", sample_rate, channels);
        Ok(())
    }

    /// Iniciar reproducción de audio (para reproducir audio del peer)
    pub fn start_playback(&mut self, device_name: Option<&str>) -> Result<(), String> {
        let device = self.select_device(device_name, false)?;
        let config = device.default_output_config()
            .map_err(|e| format!("Error al obtener configuración de salida: {}", e))?;

        let channels = config.channels();
        *self.output_channels.lock().unwrap() = channels;

        let opus_decoder = opus::Decoder::new(48000, opus::Channels::Mono)
            .map_err(|e| format!("Error al crear decodificador Opus: {:?}", e))?;
        *self.decoder.lock().unwrap() = Some(opus_decoder);

        let playback_buffer = self.playback_buffer.clone();

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut buffer = playback_buffer.lock().unwrap();
                let to_read = data.len().min(buffer.len());
                for i in 0..to_read {
                    data[i] = buffer[i];
                }
                buffer.drain(0..to_read);
                // Rellenar con silencio si no hay suficientes datos
                for i in to_read..data.len() {
                    data[i] = 0.0;
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

    /// Recibe un paquete codificado de Opus, lo decodifica y lo añade al buffer de reproducción
    pub fn play_raw_data(&self, data: Vec<u8>) -> Result<(), String> {
        let mut decoder_guard = self.decoder.lock().unwrap();
        if let Some(ref mut decoder) = *decoder_guard {
            let mut decoded = vec![0.0f32; 960]; // 20ms a 48000Hz Mono
            match decoder.decode_float(&data, &mut decoded, false) {
                Ok(len) => {
                    let mut buffer = self.playback_buffer.lock().unwrap();
                    let channels = *self.output_channels.lock().unwrap();
                    
                    // Si el dispositivo espera estéreo (2 canales), duplicamos las muestras mono
                    if channels == 2 {
                        for &sample in &decoded[0..len] {
                            buffer.push(sample); // Izquierdo
                            buffer.push(sample); // Derecho
                        }
                    } else {
                        buffer.extend_from_slice(&decoded[0..len]);
                    }
                    Ok(())
                }
                Err(e) => Err(format!("Error al decodificar audio: {:?}", e)),
            }
        } else {
            Err("Decodificador de audio no inicializado".to_string())
        }
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
        *self.decoder.lock().unwrap() = None;
        self.playback_buffer.lock().unwrap().clear();
        info!("Audio detenido");
    }
}
