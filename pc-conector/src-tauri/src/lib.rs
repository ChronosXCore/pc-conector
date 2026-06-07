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
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Manager;

/// Estado global de la aplicación compartido entre comandos Tauri
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub network: Arc<Mutex<Option<NetworkManager>>>,
    pub connections: Arc<Mutex<std::collections::HashMap<String, quinn::Connection>>>,
    pub clipboard: Arc<Mutex<Option<ClipboardSync>>>,
    pub input_service: Arc<Mutex<Option<InputService>>>,
    pub audio_service: Arc<Mutex<Option<AudioService>>>,
    pub discovery: Arc<Mutex<Option<DiscoveryService>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(AppConfig::load())),
            network: Arc::new(Mutex::new(None)),
            connections: Arc::new(Mutex::new(std::collections::HashMap::new())),
            clipboard: Arc::new(Mutex::new(None)),
            input_service: Arc::new(Mutex::new(None)),
            audio_service: Arc::new(Mutex::new(None)),
            discovery: Arc::new(Mutex::new(None)),
        }
    }
}

/// Envía un mensaje de red de forma asyncrónica a todos los peers conectados
fn send_to_all_peers(msg: NetworkMessage, state: &AppState) {
    let conns = state.connections.lock().unwrap().clone();
    for (addr, conn) in conns {
        let msg_clone = msg.clone();
        tauri::async_runtime::spawn(async move {
            let bytes = match serde_json::to_vec(&msg_clone) {
                Ok(b) => b,
                Err(e) => {
                    error!("Error al serializar mensaje para {}: {}", addr, e);
                    return;
                }
            };
            match conn.open_uni().await {
                Ok(mut send) => {
                    if let Err(e) = send.write_all(&bytes).await {
                        error!("Error al enviar mensaje por red a {}: {}", addr, e);
                    }
                    let _ = send.finish();
                }
                Err(e) => {
                    error!("Error al abrir stream uni para envío a {}: {}", addr, e);
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
        NetworkMessage::AudioData(data) => {
            if let Some(ref mut audio) = *state.audio_service.lock().unwrap() {
                if let Err(e) = audio.play_raw_data(data) {
                    error!("Error al reproducir audio remoto: {}", e);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Inicia el bucle receptor de mensajes en segundo plano para una conexión activa, gestionando la autenticación mutua por token
fn start_receive_loop(conn: quinn::Connection, state: AppState, addr: String, is_server: bool) {
    tauri::async_runtime::spawn(async move {
        info!("Iniciando bucle de recepción para peer {} (servidor: {})...", addr, is_server);
        
        let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "PC-Desconocido".to_string());
        let expected_token = state.config.lock().unwrap().connection.security_token.clone();
        
        // Si somos el cliente, enviamos nuestras credenciales inmediatamente al conectar
        if !is_server {
            send_peer_info(&conn, hostname.clone(), expected_token.clone());
        }

        let mut authenticated = false;
        
        loop {
            match receive_message_on_conn(&conn).await {
                Ok(msg) => {
                    if !authenticated {
                        if let NetworkMessage::PeerInfo(peer_host, peer_token) = msg {
                            if peer_token == expected_token {
                                info!("Autenticación exitosa con {} ({})", peer_host, addr);
                                authenticated = true;
                                
                                // Si somos el servidor, enviamos nuestras credenciales en respuesta
                                if is_server {
                                    send_peer_info(&conn, hostname.clone(), expected_token.clone());
                                }
                                continue;
                            } else {
                                warn!("Token inválido recibido de {} ({}). Cerrando conexión.", peer_host, addr);
                                conn.close(0u32.into(), b"Token de seguridad incorrecto");
                                state.connections.lock().unwrap().remove(&addr);
                                break;
                            }
                        } else {
                            warn!("Mensaje no autorizado recibido antes de autenticar de {}. Cerrando.", addr);
                            conn.close(0u32.into(), b"No autenticado");
                            state.connections.lock().unwrap().remove(&addr);
                            break;
                        }
                    }
                    
                    if let Err(e) = handle_incoming_message(msg, &state) {
                        error!("Error al procesar mensaje entrante de {}: {}", addr, e);
                    }
                }
                Err(e) => {
                    error!("Conexión de red perdida o error al recibir de {}: {}", addr, e);
                    state.connections.lock().unwrap().remove(&addr);
                    break;
                }
            }
        }
        info!("Bucle de recepción para {} finalizado.", addr);
    });
}

fn send_peer_info(conn: &quinn::Connection, hostname: String, token: String) {
    let msg = NetworkMessage::PeerInfo(hostname, token);
    let conn_clone = conn.clone();
    tauri::async_runtime::spawn(async move {
        if let Ok(bytes) = serde_json::to_vec(&msg) {
            if let Ok(mut send) = conn_clone.open_uni().await {
                let _ = send.write_all(&bytes).await;
                let _ = send.finish();
            }
        }
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
async fn start_discovery(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    info!("Iniciando descubrimiento de peers...");
    
    // Si no está iniciado en el arranque (o si fue detenido), lo iniciamos
    let has_discovery = state.discovery.lock().unwrap().is_some();
    if !has_discovery {
        let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "PC-Desconocido".to_string());
        let mut discovery = DiscoveryService::new();
        discovery.start(&hostname, 9876).map_err(|e| e.to_string())?;
        *state.discovery.lock().unwrap() = Some(discovery);
    }
    
    // Esperar 2 segundos para dar tiempo a descubrir peers adicionales en la red local sin bloquear el hilo
    tokio::time::sleep(Duration::from_secs(2)).await;
    
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

fn parse_ips_from_arp_output(output: &str) -> Vec<String> {
    let mut ips = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        if let Ok(ip) = parts[0].parse::<std::net::Ipv4Addr>() {
            if is_valid_local_ip(&ip) {
                ips.push(ip.to_string());
            }
        } else if parts.len() >= 2 {
            if let Ok(ip) = parts[1].parse::<std::net::Ipv4Addr>() {
                if is_valid_local_ip(&ip) {
                    ips.push(ip.to_string());
                }
            }
        }
    }
    ips
}

fn is_valid_local_ip(ip: &std::net::Ipv4Addr) -> bool {
    let octets = ip.octets();
    if ip.is_loopback() {
        return false;
    }
    if ip.is_multicast() {
        return false;
    }
    if octets == [255, 255, 255, 255] {
        return false;
    }
    if octets[3] == 0 || octets[3] == 255 {
        return false;
    }
    true
}

#[tauri::command]
async fn start_free_discovery() -> Result<Vec<String>, String> {
    let mut ips = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("arp").arg("-a").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            ips.extend(parse_ips_from_arp_output(&stdout));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/net/arp") {
            ips.extend(parse_ips_from_arp_output(&content));
        } else {
            if let Ok(output) = std::process::Command::new("arp").arg("-an").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                ips.extend(parse_ips_from_arp_output(&stdout));
            } else if let Ok(output) = std::process::Command::new("arp").arg("-a").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                ips.extend(parse_ips_from_arp_output(&stdout));
            }
        }
    }

    ips.sort();
    ips.dedup();

    info!("Búsqueda libre completada. IPs encontradas: {:?}", ips);
    Ok(ips)
}

#[tauri::command]
async fn connect_to_peer(addr: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    info!("Conectando a {}...", addr);
    
    let server_addr = addr.clone();
    let conn = NetworkManager::connect(&server_addr, 9876).await?;
    
    state.connections.lock().unwrap().insert(addr.clone(), conn.clone());
    
    // Iniciar bucle receptor para el cliente
    start_receive_loop(conn, state.inner().clone(), addr.clone(), false);
    
    Ok(format!("Conectado a {}", addr))
}

#[tauri::command]
fn disconnect_from_peer(addr: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut conns = state.connections.lock().unwrap();
    if let Some(conn) = conns.remove(&addr) {
        conn.close(0u32.into(), b"Desconectado por el usuario");
        info!("Desconectado de {}", addr);
    }
    Ok(())
}

#[tauri::command]
fn disconnect(state: tauri::State<AppState>) -> Result<(), String> {
    let mut conns = state.connections.lock().unwrap();
    for (addr, conn) in conns.drain() {
        conn.close(0u32.into(), b"Desconectado por el usuario");
        info!("Desconectado de {}", addr);
    }
    Ok(())
}

#[tauri::command]
fn get_connection_status(state: tauri::State<AppState>) -> bool {
    !state.connections.lock().unwrap().is_empty()
}

#[tauri::command]
fn get_connected_peers(state: tauri::State<AppState>) -> Vec<String> {
    state.connections.lock().unwrap().keys().cloned().collect()
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
        send_to_all_peers(NetworkMessage::Clipboard(text), &state_clone);
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
        send_to_all_peers(msg, &state_clone);
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
    
    let state_clone = state.inner().clone();
    audio.set_on_encoded_data(move |encoded_bytes| {
        send_to_all_peers(NetworkMessage::AudioData(encoded_bytes), &state_clone);
    });

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
    let _ = rustls::crypto::ring::default_provider().install_default();
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .setup(|app| {
            let state = app.state::<AppState>();
            let state_clone = state.inner().clone();
            
            // Iniciar descubrimiento en segundo plano y registrar el servicio mDNS al arrancar
            let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "PC-Desconocido".to_string());
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
                let endpoint = match network_mgr.start_server(9876).await {
                    Ok(ep) => ep,
                    Err(e) => {
                        error!("Error al iniciar servidor QUIC: {}", e);
                        return;
                    }
                };
                
                *state_clone.network.lock().unwrap() = Some(network_mgr);
                info!("Servidor QUIC iniciado en puerto 9876. Esperando conexiones...");
                
                loop {
                    // Aceptar conexiones entrantes sin mantener el lock
                    match endpoint.accept().await {
                        Some(incoming) => {
                            let state_nested = state_clone.clone();
                            tauri::async_runtime::spawn(async move {
                                match incoming.await {
                                    Ok(conn) => {
                                        let remote_addr = conn.remote_address().to_string();
                                        info!("Conexión entrante aceptada de {}", remote_addr);
                                        
                                        // Guardar conexión en la lista de conexiones activas
                                        state_nested.connections.lock().unwrap().insert(remote_addr.clone(), conn.clone());
                                        
                                        // Iniciar bucle receptor independiente para esta conexión
                                        start_receive_loop(conn, state_nested, remote_addr, true);
                                    }
                                    Err(e) => {
                                        error!("Error al aceptar conexión QUIC entrante: {}", e);
                                    }
                                }
                            });
                        }
                        None => {
                            error!("Endpoint QUIC cerrado");
                            break;
                        }
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_discovery,
            start_free_discovery,
            connect_to_peer,
            disconnect_from_peer,
            disconnect,
            get_connection_status,
            get_connected_peers,
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
