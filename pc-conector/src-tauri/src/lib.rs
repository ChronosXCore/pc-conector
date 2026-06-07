mod audio;
mod clipboard;
mod config;
mod discovery;
mod input;
mod network;

use audio::AudioService;
use clipboard::ClipboardSync;
use config::AppConfig;
use discovery::DiscoveryService;
use input::{InputService, InputEvent};
use network::{NetworkManager, NetworkMessage};
use log::{info, error};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Manager;

/// Estado global de la aplicación compartido entre comandos Tauri
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub network: Arc<Mutex<Option<NetworkManager>>>,
    pub connection: Arc<Mutex<Option<quinn::Connection>>>,
    pub clipboard: Arc<Mutex<Option<ClipboardSync>>>,
    pub input_service: Arc<Mutex<Option<InputService>>>,
    pub audio_service: Arc<Mutex<Option<AudioService>>>,
    pub discovery: Arc<Mutex<Option<DiscoveryService>>>,
    pub is_connected: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(AppConfig::load())),
            network: Arc::new(Mutex::new(None)),
            connection: Arc::new(Mutex::new(None)),
            clipboard: Arc::new(Mutex::new(None)),
            input_service: Arc::new(Mutex::new(None)),
            audio_service: Arc::new(Mutex::new(None)),
            discovery: Arc::new(Mutex::new(None)),
            is_connected: Arc::new(Mutex::new(false)),
        }
    }
}

/// Envía un mensaje de red de forma asíncrona al peer conectado
fn send_to_peer(msg: NetworkMessage, state: &AppState) {
    let conn_opt = state.connection.lock().unwrap().clone();
    if let Some(conn) = conn_opt {
        tauri::async_runtime::spawn(async move {
            let bytes = match serde_json::to_vec(&msg) {
                Ok(b) => b,
                Err(e) => {
                    error!("Error al serializar mensaje: {}", e);
                    return;
                }
            };
            match conn.open_uni().await {
                Ok(mut send) => {
                    if let Err(e) = send.write_all(&bytes).await {
                        error!("Error al enviar mensaje por red: {}", e);
                    }
                    send.finish();
                }
                Err(e) => {
                    error!("Error al abrir stream uni para envío: {}", e);
                }
            }
        });
    }
}

/// Procesa un mensaje de red recibido desde el peer
fn handle_incoming_message(msg: NetworkMessage, state: &AppState) -> Result<(), String> {
    match msg {
        NetworkMessage::Clipboard(text) => {
            info!("Recibido portapapeles remoto: {}", text);
            if let Some(ref clipboard) = *state.clipboard.lock().unwrap() {
                clipboard.write(&text)?;
            }
        }
        NetworkMessage::MouseEvent(data) => {
            if let Some(ref input) = *state.input_service.lock().unwrap() {
                input.simulate_mouse(&data)?;
            }
        }
        NetworkMessage::KeyboardEvent(data) => {
            if let Some(ref input) = *state.input_service.lock().unwrap() {
                input.simulate_keyboard(&data)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Inicia el bucle receptor de mensajes en segundo plano para una conexión activa
fn start_receive_loop(state: AppState) {
    tauri::async_runtime::spawn(async move {
        info!("Iniciando bucle de recepción de mensajes de red...");
        loop {
            if !*state.is_connected.lock().unwrap() {
                break;
            }
            let conn_opt = state.connection.lock().unwrap().clone();
            if let Some(conn) = conn_opt {
                match receive_message_on_conn(&conn).await {
                    Ok(msg) => {
                        if let Err(e) = handle_incoming_message(msg, &state) {
                            error!("Error al procesar mensaje entrante: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Conexión de red perdida o error al recibir: {}", e);
                        *state.is_connected.lock().unwrap() = false;
                        *state.connection.lock().unwrap() = None;
                        break;
                    }
                }
            } else {
                break;
            }
        }
        info!("Bucle de recepción finalizado.");
    });
}

/// Acepta un stream unidireccional y deserializa el mensaje de red entrante
async fn receive_message_on_conn(conn: &quinn::Connection) -> Result<NetworkMessage, String> {
    let mut recv = conn
        .accept_uni()
        .await
        .map_err(|e| format!("Error al aceptar stream uni: {}", e))?;

    let data = recv
        .read_to_end(usize::MAX)
        .await
        .map_err(|e| format!("Error al leer: {}", e))?;

    let msg: NetworkMessage = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    Ok(msg)
}

#[tauri::command]
fn start_discovery(state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    info!("Iniciando descubrimiento de peers...");
    
    // Si no está iniciado en el arranque (o si fue detenido), lo iniciamos
    let has_discovery = state.discovery.lock().unwrap().is_some();
    if !has_discovery {
        let hostname = whoami::hostname();
        let mut discovery = DiscoveryService::new();
        discovery.start(&hostname, 9876).map_err(|e| e.to_string())?;
        *state.discovery.lock().unwrap() = Some(discovery);
    }
    
    // Esperar 2 segundos para dar tiempo a descubrir peers adicionales en la red local
    std::thread::sleep(Duration::from_secs(2));
    
    let discovery_guard = state.discovery.lock().unwrap();
    let peers = if let Some(ref discovery) = *discovery_guard {
        discovery.get_peers()
    } else {
        Vec::new()
    };
    
    let peer_names: Vec<String> = peers
        .iter()
        .map(|p| format!("{} - {}", p.name, p.ip_address))
        .collect();
    
    info!("Peers encontrados ({}): {:?}", peer_names.len(), peer_names);
    
    Ok(peer_names)
}

#[tauri::command]
fn connect_to_peer(addr: String, state: tauri::State<AppState>) -> Result<String, String> {
    info!("Conectando a {}...", addr);
    
    let mut network = NetworkManager::new(state.config.lock().unwrap().clone());
    let server_addr = addr.clone();
    
    tauri::async_runtime::block_on(async {
        network.connect(&server_addr, 9876).await
    })?;
    
    let conn = network.connection.clone().ok_or("No se pudo extraer la conexión")?;
    
    *state.network.lock().unwrap() = Some(network);
    *state.connection.lock().unwrap() = Some(conn);
    *state.is_connected.lock().unwrap() = true;
    
    // Iniciar bucle receptor para el cliente
    start_receive_loop(state.inner().clone());
    
    Ok(format!("Conectado a {}", addr))
}

#[tauri::command]
fn disconnect(state: tauri::State<AppState>) -> Result<(), String> {
    if let Some(ref mut network) = *state.network.lock().unwrap() {
        network.disconnect();
    }
    *state.connection.lock().unwrap() = None;
    *state.is_connected.lock().unwrap() = false;
    info!("Desconectado");
    Ok(())
}

#[tauri::command]
fn get_connection_status(state: tauri::State<AppState>) -> bool {
    *state.is_connected.lock().unwrap()
}

#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn update_config(config: AppConfig, state: tauri::State<AppState>) -> Result<(), String> {
    config.save()?;
    *state.config.lock().unwrap() = config;
    Ok(())
}

#[tauri::command]
fn start_clipboard_sync(state: tauri::State<AppState>) -> Result<(), String> {
    let mut clipboard = ClipboardSync::new();
    clipboard.init()?;
    
    let state_clone = state.inner().clone();
    clipboard.set_on_change(move |text| {
        info!("Enviando portapapeles local a remoto...");
        send_to_peer(NetworkMessage::Clipboard(text), &state_clone);
    });
    
    clipboard.start_monitoring();
    *state.clipboard.lock().unwrap() = Some(clipboard);
    Ok(())
}

#[tauri::command]
fn start_input_capture(state: tauri::State<AppState>) -> Result<(), String> {
    let mut input = InputService::new();
    
    let state_clone = state.inner().clone();
    input.set_on_input(move |event| {
        let msg = match event {
            InputEvent::MouseMove { x, y } => NetworkMessage::MouseEvent(crate::network::MouseData {
                event_type: crate::network::MouseEventType::Move,
                x,
                y,
                button: None,
                scroll_delta: None,
            }),
            InputEvent::MousePress { button } => NetworkMessage::MouseEvent(crate::network::MouseData {
                event_type: crate::network::MouseEventType::Press,
                x: 0.0,
                y: 0.0,
                button: Some(button),
                scroll_delta: None,
            }),
            InputEvent::MouseRelease { button } => NetworkMessage::MouseEvent(crate::network::MouseData {
                event_type: crate::network::MouseEventType::Release,
                x: 0.0,
                y: 0.0,
                button: Some(button),
                scroll_delta: None,
            }),
            InputEvent::MouseScroll { delta_x, delta_y } => NetworkMessage::MouseEvent(crate::network::MouseData {
                event_type: crate::network::MouseEventType::Scroll,
                x: 0.0,
                y: 0.0,
                button: None,
                scroll_delta: Some((delta_x, delta_y)),
            }),
            InputEvent::KeyPress { key: _, char } => NetworkMessage::KeyboardEvent(crate::network::KeyboardData {
                event_type: crate::network::KeyboardEventType::Press,
                key_code: 0,
                key_char: char,
            }),
            InputEvent::KeyRelease { key: _, char } => NetworkMessage::KeyboardEvent(crate::network::KeyboardData {
                event_type: crate::network::KeyboardEventType::Release,
                key_code: 0,
                key_char: char,
            }),
        };
        send_to_peer(msg, &state_clone);
    });
    
    input.start_capture()?;
    *state.input_service.lock().unwrap() = Some(input);
    Ok(())
}

#[tauri::command]
fn start_audio_capture(state: tauri::State<AppState>) -> Result<(), String> {
    let mut audio = AudioService::new();
    let config = state.config.lock().unwrap();
    
    let input_device = config.audio.input_device.as_deref();
    let output_device = config.audio.output_device.as_deref();
    
    if config.audio.stream_microphone {
        audio.start_capture(input_device)?;
    }
    if config.audio.stream_speakers {
        audio.start_playback(output_device)?;
    }
    
    *state.audio_service.lock().unwrap() = Some(audio);
    Ok(())
}

#[tauri::command]
fn list_audio_devices(_state: tauri::State<AppState>) -> Result<(Vec<audio::AudioDeviceInfo>, Vec<audio::AudioDeviceInfo>), String> {
    let audio = AudioService::new();
    audio.list_devices()
}

#[tauri::command]
fn stop_services(state: tauri::State<AppState>) -> Result<(), String> {
    if let Some(ref clipboard) = *state.clipboard.lock().unwrap() {
        clipboard.stop_monitoring();
    }
    if let Some(ref input) = *state.input_service.lock().unwrap() {
        input.stop_capture();
    }
    if let Some(ref mut audio) = *state.audio_service.lock().unwrap() {
        audio.stop();
    }
    *state.clipboard.lock().unwrap() = None;
    *state.input_service.lock().unwrap() = None;
    *state.audio_service.lock().unwrap() = None;
    
    disconnect(state)?;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .setup(|app| {
            let state = app.state::<AppState>();
            let state_clone = state.inner().clone();
            
            // Iniciar descubrimiento en segundo plano y registrar el servicio mDNS al arrancar
            let hostname = whoami::hostname();
            let mut discovery = DiscoveryService::new();
            match discovery.start(&hostname, 9876) {
                Ok(()) => {
                    *state.discovery.lock().unwrap() = Some(discovery);
                    info!("Servicio de descubrimiento mDNS iniciado en el arranque para {}", hostname);
                }
                Err(e) => {
                    error!("Error al iniciar servicio de descubrimiento mDNS en el arranque: {}", e);
                }
            }
            
            // Iniciar servidor QUIC en puerto 9876 en segundo plano para escuchar conexiones de otros equipos
            tauri::async_runtime::spawn(async move {
                let mut network_mgr = NetworkManager::new(state_clone.config.lock().unwrap().clone());
                if let Err(e) = network_mgr.start_server(9876).await {
                    error!("Error al iniciar servidor QUIC: {}", e);
                    return;
                }
                
                let endpoint = network_mgr.endpoint.clone().unwrap();
                *state_clone.network.lock().unwrap() = Some(network_mgr);
                info!("Servidor QUIC iniciado en puerto 9876. Esperando conexiones...");
                
                loop {
                    // Aceptar conexiones entrantes sin mantener el lock
                    match endpoint.accept().await {
                        Some(incoming) => {
                            match incoming.await {
                                Ok(conn) => {
                                    info!("Conexión entrante aceptada de {}", conn.remote_address());
                                    
                                    // Guardar conexión en el network manager y el estado
                                    let mut net_guard = state_clone.network.lock().unwrap();
                                    if let Some(ref mut net) = *net_guard {
                                        net.connection = Some(conn.clone());
                                    }
                                    *state_clone.connection.lock().unwrap() = Some(conn.clone());
                                    *state_clone.is_connected.lock().unwrap() = true;
                                    
                                    // Iniciar bucle receptor para el servidor
                                    start_receive_loop(state_clone.clone());
                                }
                                Err(e) => {
                                    error!("Error al aceptar conexión QUIC entrante: {}", e);
                                }
                            }
                        }
                        None => {
                            error!("Endpoint QUIC cerrado");
                            break;
                        }
                    }
                    
                    // Esperar a que se desconecte antes de volver a escuchar
                    while *state_clone.is_connected.lock().unwrap() {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_discovery,
            connect_to_peer,
            disconnect,
            get_connection_status,
            get_config,
            update_config,
            start_clipboard_sync,
            start_input_capture,
            start_audio_capture,
            list_audio_devices,
            stop_services,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
