mod audio;
mod clipboard;
mod config;
mod discovery;
mod input;
mod network;

use audio::AudioService;
use clipboard::ClipboardSync;
use config::{AppConfig, LinkedDevice};
use discovery::DiscoveryService;
use input::{InputService, InputEvent};
use network::{NetworkManager, NetworkMessage, ScreenInfo};
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Manager;
use tauri::Emitter;
use tauri::tray::TrayIconBuilder;

// ───────────────────────────────────────────────────────────────────────────
// Virtual layout entry: a screen (local OR remote) placed on the shared canvas
// ───────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VirtualScreen {
    pub id: String,
    pub name: String,
    pub owner: String,   // "local" or peer IP
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

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
    /// Screens reported by remote peers: addr -> list of ScreenInfo
    pub remote_screens: Arc<Mutex<std::collections::HashMap<String, Vec<ScreenInfo>>>>,
    /// Audio devices reported by remote peers: addr -> (inputs, outputs)
    pub remote_audio_devices: Arc<Mutex<std::collections::HashMap<String, (Vec<audio::AudioDeviceInfo>, Vec<audio::AudioDeviceInfo>)>>>,
    /// Whether cursor is currently locked to remote (input forwarding active)
    pub cursor_on_remote: Arc<Mutex<bool>>,
    pub app_handle: Arc<Mutex<Option<tauri::AppHandle>>>,
    pub pending_approvals: Arc<Mutex<std::collections::HashMap<String, tokio::sync::oneshot::Sender<bool>>>>,
    /// Virtual layout: combined local+remote screens placed on a 2D canvas
    pub virtual_layout: Arc<Mutex<Vec<VirtualScreen>>>,
    /// Peer we are currently forwarding input to (if any)
    pub forwarding_to: Arc<Mutex<Option<String>>>,
    /// Last known raw mouse position (absolute pixel coords on local desktop)
    pub last_mouse_pos: Arc<Mutex<(f64, f64)>>,
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
            remote_screens: Arc::new(Mutex::new(std::collections::HashMap::new())),
            remote_audio_devices: Arc::new(Mutex::new(std::collections::HashMap::new())),
            cursor_on_remote: Arc::new(Mutex::new(false)),
            app_handle: Arc::new(Mutex::new(None)),
            pending_approvals: Arc::new(Mutex::new(std::collections::HashMap::new())),
            virtual_layout: Arc::new(Mutex::new(Vec::new())),
            forwarding_to: Arc::new(Mutex::new(None)),
            last_mouse_pos: Arc::new(Mutex::new((0.0, 0.0))),
        }
    }
}

fn emit_connections_changed(state: &AppState) {
    if let Some(ref handle) = *state.app_handle.lock().unwrap() {
        let active_peers: Vec<String> = state.connections.lock().unwrap().keys().cloned().collect();
        let _ = handle.emit("connections-changed", active_peers);
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

/// Envía un mensaje de red a un peer específico
fn send_to_peer(peer_addr: &str, msg: NetworkMessage, state: &AppState) {
    let conn = {
        let conns = state.connections.lock().unwrap();
        conns.get(peer_addr).cloned().or_else(|| {
            conns.keys()
                .find(|k| k.starts_with(peer_addr))
                .and_then(|k| conns.get(k).cloned())
        })
    };
    if let Some(conn) = conn {
        let addr = peer_addr.to_string();
        tauri::async_runtime::spawn(async move {
            let bytes = match serde_json::to_vec(&msg) {
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

// ───────────────────────────────────────────────────────────────────────────
// KVM EDGE DETECTION
// Given the current mouse position and the virtual layout, detect if the
// cursor has crossed into a remote screen. Returns the peer IP and the
// normalised (0..1) position within the remote screen if so.
// ───────────────────────────────────────────────────────────────────────────
fn find_remote_screen_at(x: f64, y: f64, layout: &[VirtualScreen]) -> Option<(String, f64, f64)> {
    for vs in layout {
        if vs.owner == "local" {
            continue;
        }
        let rx = vs.x as f64;
        let ry = vs.y as f64;
        let rw = vs.width as f64;
        let rh = vs.height as f64;
        if x >= rx && x < rx + rw && y >= ry && y < ry + rh {
            let nx = (x - rx) / rw;
            let ny = (y - ry) / rh;
            return Some((vs.owner.clone(), nx, ny));
        }
    }
    None
}

/// Returns the primary local screen from the virtual layout, or None
fn primary_local_screen(layout: &[VirtualScreen]) -> Option<&VirtualScreen> {
    layout.iter().find(|v| v.owner == "local" && v.is_primary)
        .or_else(|| layout.iter().find(|v| v.owner == "local"))
}

/// Called on every mouse-move event from rdev. Checks edge-crossing and
/// decides whether to forward to remote or keep local.
fn handle_mouse_move(x: f64, y: f64, state: &AppState) {
    *state.last_mouse_pos.lock().unwrap() = (x, y);

    let layout = state.virtual_layout.lock().unwrap().clone();
    if layout.is_empty() {
        return;
    }

    let currently_forwarding = state.forwarding_to.lock().unwrap().clone();

    if let Some(peer_ip) = currently_forwarding {
        // We are already forwarding — find the remote screen for this peer and
        // check if the cursor has re-entered a LOCAL screen.
        let remote_vs: Vec<&VirtualScreen> = layout.iter().filter(|v| v.owner == peer_ip).collect();
        
        // Check if we're back in a local area
        let in_local = layout.iter().any(|v| {
            if v.owner != "local" { return false; }
            let rx = v.x as f64; let ry = v.y as f64;
            let rw = v.width as f64; let rh = v.height as f64;
            x >= rx && x < rx + rw && y >= ry && y < ry + rh
        });

        if in_local && !remote_vs.is_empty() {
            // Return control to local
            *state.forwarding_to.lock().unwrap() = None;
            *state.cursor_on_remote.lock().unwrap() = false;
            info!("Cursor devuelto al equipo local desde {}", peer_ip);
            if let Some(ref handle) = *state.app_handle.lock().unwrap() {
                let _ = handle.emit("cursor-returned-local", ());
            }
        } else {
            // Still in remote territory — compute normalised position within the remote screen
            if let Some(rem) = remote_vs.first() {
                let rx = rem.x as f64; let ry = rem.y as f64;
                let rw = rem.width as f64; let rh = rem.height as f64;
                let nx = ((x - rx) / rw).clamp(0.0, 1.0);
                let ny = ((y - ry) / rh).clamp(0.0, 1.0);
                // Send normalised move to remote peer
                let msg = NetworkMessage::MouseEvent(network::MouseData {
                    event_type: network::MouseEventType::Move,
                    x: nx,
                    y: ny,
                    button: None,
                    scroll_delta: None,
                });
                send_to_peer(&peer_ip, msg, state);
            }
        }
        return;
    }

    // Not forwarding — check if cursor has moved into a remote screen zone
    if let Some((peer_ip, nx, ny)) = find_remote_screen_at(x, y, &layout) {
        // Transition: start forwarding to peer
        *state.forwarding_to.lock().unwrap() = Some(peer_ip.clone());
        *state.cursor_on_remote.lock().unwrap() = true;
        info!("Cursor entró en pantalla remota de {}", peer_ip);

        // Snap cursor back to the edge of the nearest local screen so it disappears
        if let Some(local) = primary_local_screen(&layout) {
            // Use enigo to move cursor back to center of primary local screen
            let center_x = local.x + (local.width as i32 / 2);
            let center_y = local.y + (local.height as i32 / 2);
            if let Some(ref svc) = *state.input_service.lock().unwrap() {
                let _ = svc.warp_mouse(center_x, center_y);
            }
        }

        // Notify frontend
        if let Some(ref handle) = *state.app_handle.lock().unwrap() {
            let _ = handle.emit("cursor-on-remote", peer_ip.clone());
        }

        // Send move to remote
        let msg = NetworkMessage::MouseEvent(network::MouseData {
            event_type: network::MouseEventType::Move,
            x: nx,
            y: ny,
            button: None,
            scroll_delta: None,
        });
        send_to_peer(&peer_ip, msg, state);
    }
}

/// Procesa un mensaje de red recibido desde el peer
fn handle_incoming_message(msg: NetworkMessage, state: &AppState, peer_addr: &str) -> Result<(), String> {
    match msg {
        NetworkMessage::Clipboard(text) => {
            info!("Recibido portapapeles remoto: {}", text);
            if let Some(ref clipboard) = *state.clipboard.lock().unwrap() {
                clipboard.write(&text)?;
            }
        }
        NetworkMessage::MouseEvent(data) => {
            // Check if this is a relative (normalised 0..1) move coming from remote
            if let network::MouseEventType::Move = data.event_type {
                // If x,y are in 0..1 range they are normalised remote-to-local coords
                if data.x >= 0.0 && data.x <= 1.0 && data.y >= 0.0 && data.y <= 1.0 {
                    // Map to our local primary screen
                    let local_screens = collect_local_screens();
                    if let Some(primary) = local_screens.iter().find(|s| s.is_primary).or(local_screens.first()) {
                        let abs_x = primary.x as f64 + data.x * primary.width as f64;
                        let abs_y = primary.y as f64 + data.y * primary.height as f64;
                        if let Some(ref input) = *state.input_service.lock().unwrap() {
                            let abs_data = network::MouseData {
                                event_type: network::MouseEventType::Move,
                                x: abs_x,
                                y: abs_y,
                                button: None,
                                scroll_delta: None,
                            };
                            let _ = input.simulate_mouse(&abs_data);
                        }
                        return Ok(());
                    }
                }
            }
            // Otherwise simulate as-is
            if let Some(ref input) = *state.input_service.lock().unwrap() {
                input.simulate_mouse(&data)?;
            }
        }
        NetworkMessage::KeyboardEvent(data) => {
            if let Some(ref input) = *state.input_service.lock().unwrap() {
                input.simulate_keyboard(&data)?;
            }
        }
        NetworkMessage::AudioData { route_id: _, data } => {
            if let Some(ref mut audio) = *state.audio_service.lock().unwrap() {
                if let Err(e) = audio.play_raw_data(data) {
                    error!("Error al reproducir audio remoto: {}", e);
                }
            }
        }
        NetworkMessage::StartAudioCapture { device_name } => {
            info!("Petición remota de iniciar captura en: {}", device_name);
            let state_clone = state.clone();
            let peer_addr_clone = peer_addr.to_string();
            
            if let Some(ref mut audio) = *state.audio_service.lock().unwrap() {
                audio.stop();
            }

            let mut audio = AudioService::new();
            audio.set_on_encoded_data(move |bytes| {
                let conn_addr = {
                    let conns = state_clone.connections.lock().unwrap();
                    conns.keys().find(|k| k.starts_with(&peer_addr_clone)).cloned()
                };
                if let Some(addr) = conn_addr {
                    send_to_peer(&addr, NetworkMessage::AudioData { route_id: "patchbay".to_string(), data: bytes }, &state_clone);
                }
            });
            if let Err(e) = audio.start_capture(Some(&device_name)) {
                error!("Error al iniciar captura remota: {}", e);
            } else {
                *state.audio_service.lock().unwrap() = Some(audio);
            }
        }
        NetworkMessage::StartAudioPlayback { device_name } => {
            info!("Petición remota de iniciar reproducción en: {}", device_name);
            if let Some(ref mut audio) = *state.audio_service.lock().unwrap() {
                audio.stop();
            }
            let mut audio = AudioService::new();
            if let Err(e) = audio.start_playback(Some(&device_name)) {
                error!("Error al iniciar reproducción remota: {}", e);
            } else {
                *state.audio_service.lock().unwrap() = Some(audio);
            }
        }
        NetworkMessage::StopAudioCapture => {
            info!("Petición remota de detener captura");
            if let Some(ref mut audio) = *state.audio_service.lock().unwrap() {
                audio.stop();
            }
        }
        NetworkMessage::StopAudioPlayback => {
            info!("Petición remota de detener reproducción");
            if let Some(ref mut audio) = *state.audio_service.lock().unwrap() {
                audio.stop();
            }
        }
        NetworkMessage::AudioDevices { inputs, outputs } => {
            info!("Recibidos dispositivos de audio de {}: inputs={}, outputs={}", peer_addr, inputs.len(), outputs.len());
            state.remote_audio_devices.lock().unwrap().insert(peer_addr.to_string(), (inputs, outputs));
            if let Some(ref handle) = *state.app_handle.lock().unwrap() {
                let _ = handle.emit("remote-audio-devices-changed", ());
            }
        }
        NetworkMessage::ScreenLayout(screens) => {
            info!("Recibido layout de pantallas de {}: {} pantalla(s)", peer_addr, screens.len());
            state.remote_screens.lock().unwrap().insert(peer_addr.to_string(), screens.clone());
            // Rebuild virtual layout
            rebuild_virtual_layout(state);
        }
        _ => {}
    }
    Ok(())
}

/// Rebuild the virtual_layout from current local+remote screens
fn rebuild_virtual_layout(state: &AppState) {
    let local = collect_local_screens();
    let remote = state.remote_screens.lock().unwrap().clone();
    let mut layout: Vec<VirtualScreen> = local.iter().map(|s| VirtualScreen {
        id: s.id.clone(),
        name: s.name.clone(),
        owner: "local".to_string(),
        x: s.x,
        y: s.y,
        width: s.width,
        height: s.height,
        is_primary: s.is_primary,
    }).collect();

    // Place remote screens to the right of all local screens
    let max_local_x = local.iter().map(|s| s.x + s.width as i32).max().unwrap_or(1920);
    let mut remote_offset_x = max_local_x;
    for (peer, screens) in &remote {
        for s in screens {
            layout.push(VirtualScreen {
                id: format!("{}-{}", peer, s.id),
                name: s.name.clone(),
                owner: peer.clone(),
                x: remote_offset_x + s.x,
                y: s.y,
                width: s.width,
                height: s.height,
                is_primary: s.is_primary,
            });
        }
        // Advance offset for next peer
        remote_offset_x += screens.iter().map(|s| s.width as i32).sum::<i32>();
    }

    *state.virtual_layout.lock().unwrap() = layout;
}

/// Inicia el bucle receptor de mensajes en segundo plano para una conexión activa
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
                                // Use a block so the MutexGuard is dropped before any await
                                let clean_ip = addr.split(':').next().unwrap_or(&addr).to_string();
                                let (require_app, is_pre_approved) = {
                                    let config = state.config.lock().unwrap();
                                    let ra = config.connection.require_approval;
                                    let ipa = config.connection.allowed_devices.contains(&clean_ip)
                                        || config.connection.allowed_devices.contains(&peer_host);
                                    (ra, ipa)
                                }; // config guard dropped here

                                let approved = if !is_server || !require_app || is_pre_approved {
                                    true
                                } else {
                                    // Emitir evento para el frontend
                                    #[derive(Clone, serde::Serialize)]
                                    struct RequestPayload {
                                        ip: String,
                                        hostname: String,
                                    }
                                    // Emit event — extract cloned handle, drop guard before await
                                    {
                                        let guard = state.app_handle.lock().unwrap();
                                        if let Some(ref handle) = *guard {
                                            let _ = handle.emit("connection-request", RequestPayload {
                                                ip: clean_ip.clone(),
                                                hostname: peer_host.clone(),
                                            });
                                        }
                                    } // guard dropped here

                                    // Crear canal oneshot y esperar aprobación
                                    let (tx, rx) = tokio::sync::oneshot::channel();
                                    {
                                        state.pending_approvals.lock().unwrap().insert(clean_ip.clone(), tx);
                                    } // guard dropped here
                                    
                                    info!("Esperando aprobación de conexión para {} ({})", peer_host, clean_ip);
                                    
                                    // Timeout of 60s — no MutexGuards held across this await
                                    match tokio::time::timeout(Duration::from_secs(60), rx).await {
                                        Ok(Ok(val)) => val,
                                        _ => false,
                                    }
                                };

                                if approved {
                                    info!("Autenticación exitosa con {} ({})", peer_host, addr);
                                    authenticated = true;
                                    
                                    // Agregar automáticamente a dispositivos vinculados (linked_devices) de forma bidireccional
                                    {
                                        let mut config = state.config.lock().unwrap();
                                        if !config.linked_devices.iter().any(|d| d.ip == clean_ip) {
                                            let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                                            config.linked_devices.push(LinkedDevice {
                                                ip: clean_ip.clone(),
                                                name: peer_host.clone(),
                                                linked_at: ts,
                                            });
                                            let _ = config.save();
                                            info!("Dispositivo {} ({}) vinculado automáticamente tras conexión exitosa", peer_host, clean_ip);
                                        }
                                    }
                                    
                                    // Si somos el servidor, enviamos nuestras credenciales en respuesta
                                    if is_server {
                                        send_peer_info(&conn, hostname.clone(), expected_token.clone());
                                    }
                                    
                                    // Enviar nuestro layout de pantallas al peer recién autenticado
                                    let local_screens = collect_local_screens();
                                    let layout_msg = NetworkMessage::ScreenLayout(local_screens);
                                    let conn_clone = conn.clone();
                                    tauri::async_runtime::spawn(async move {
                                        if let Ok(bytes) = serde_json::to_vec(&layout_msg) {
                                            if let Ok(mut send) = conn_clone.open_uni().await {
                                                let _ = send.write_all(&bytes).await;
                                                let _ = send.finish();
                                            }
                                        }
                                    });
                                    send_audio_devices(&conn);
                                    
                                    emit_connections_changed(&state);
                                    // Rebuild virtual layout now that we have a new peer
                                    rebuild_virtual_layout(&state);
                                    continue;
                                } else {
                                    warn!("Conexión rechazada para {} ({})", peer_host, addr);
                                    conn.close(0u32.into(), b"Conexion rechazada");
                                    state.connections.lock().unwrap().remove(&addr);
                                    state.remote_screens.lock().unwrap().remove(&addr);
                                    state.remote_audio_devices.lock().unwrap().remove(&addr);
                                    emit_connections_changed(&state);
                                    rebuild_virtual_layout(&state);
                                    break;
                                }
                            } else {
                                warn!("Token inválido recibido de {} ({}). Cerrando conexión.", peer_host, addr);
                                conn.close(0u32.into(), b"Token de seguridad incorrecto");
                                state.connections.lock().unwrap().remove(&addr);
                                state.remote_screens.lock().unwrap().remove(&addr);
                                state.remote_audio_devices.lock().unwrap().remove(&addr);
                                emit_connections_changed(&state);
                                break;
                            }
                        } else {
                            warn!("Mensaje no autorizado recibido antes de autenticar de {}. Cerrando.", addr);
                            conn.close(0u32.into(), b"No autenticado");
                            state.connections.lock().unwrap().remove(&addr);
                            state.remote_screens.lock().unwrap().remove(&addr);
                            state.remote_audio_devices.lock().unwrap().remove(&addr);
                            emit_connections_changed(&state);
                            break;
                        }
                    }
                    
                    if let Err(e) = handle_incoming_message(msg, &state, &addr) {
                        error!("Error al procesar mensaje entrante de {}: {}", addr, e);
                    }
                }
                Err(e) => {
                    error!("Conexión de red perdida o error al recibir de {}: {}", addr, e);
                    state.connections.lock().unwrap().remove(&addr);
                    state.remote_screens.lock().unwrap().remove(&addr);
                    state.remote_audio_devices.lock().unwrap().remove(&addr);
                    *state.forwarding_to.lock().unwrap() = None;
                    *state.cursor_on_remote.lock().unwrap() = false;
                    emit_connections_changed(&state);
                    rebuild_virtual_layout(&state);
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
    
    let has_discovery = state.discovery.lock().unwrap().is_some();
    if !has_discovery {
        let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "PC-Desconocido".to_string());
        let mut discovery = DiscoveryService::new();
        discovery.start(&hostname, 9876).map_err(|e| e.to_string())?;
        *state.discovery.lock().unwrap() = Some(discovery);
    }
    
    // Wait 3 seconds for mDNS + UDP broadcast peers to respond
    tokio::time::sleep(Duration::from_secs(3)).await;
    
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
async fn get_discovered_peers(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let discovery_guard = state.discovery.lock().unwrap();
    let peers = if let Some(ref discovery) = *discovery_guard {
        discovery.get_peers()
    } else {
        Vec::new()
    };
    Ok(peers.iter().map(|p| format!("{} - {}", p.name, p.ip_address)).collect())
}

fn is_valid_local_ip(ip: &std::net::Ipv4Addr) -> bool {
    let octets = ip.octets();
    if ip.is_loopback() { return false; }
    if ip.is_multicast() { return false; }
    if octets == [255, 255, 255, 255] { return false; }
    if octets[3] == 0 || octets[3] == 255 { return false; }
    true
}

fn configure_system_firewall() {
    #[cfg(target_os = "linux")]
    {
        info!("Intentando configurar cortafuegos en Linux...");
        if std::process::Command::new("ufw").arg("--version").status().is_ok() {
            let status = std::process::Command::new("ufw").arg("status").output();
            if let Ok(out) = status {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.contains("active") || stdout.contains("activo") {
                    info!("UFW detectado activo. Habilitando puertos de red para NetBridge...");
                    let _ = std::process::Command::new("pkexec").args(["ufw", "allow", "9876/udp"]).status();
                    let _ = std::process::Command::new("pkexec").args(["ufw", "allow", "9875/udp"]).status();
                    let _ = std::process::Command::new("pkexec").args(["ufw", "allow", "5353/udp"]).status();
                }
            }
        }
        if std::process::Command::new("firewall-cmd").arg("--version").status().is_ok() {
            let status = std::process::Command::new("firewall-cmd").arg("--state").output();
            if let Ok(out) = status {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if stdout == "running" {
                    let _ = std::process::Command::new("pkexec").args(["firewall-cmd", "--add-port=9876/udp", "--permanent"]).status();
                    let _ = std::process::Command::new("pkexec").args(["firewall-cmd", "--add-port=9875/udp", "--permanent"]).status();
                    let _ = std::process::Command::new("pkexec").args(["firewall-cmd", "--reload"]).status();
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        info!("Intentando configurar cortafuegos en Windows...");
        let _ = std::process::Command::new("powershell")
            .args([
                "-NoProfile", "-WindowStyle", "Hidden", "-Command",
                "Start-Process netsh -ArgumentList 'advfirewall firewall add rule name=\"NetBridge\" dir=in action=allow protocol=UDP localport=9876,9875,5353' -Verb RunAs -ErrorAction SilentlyContinue"
            ])
            .status();
    }
}

fn get_all_local_ips() -> Vec<std::net::Ipv4Addr> {
    let mut ips = Vec::new();
    if let Ok(interfaces) = get_if_addrs::get_if_addrs() {
        for iface in interfaces {
            if !iface.is_loopback() {
                if let std::net::IpAddr::V4(ip) = iface.ip() {
                    if !ips.contains(&ip) { ips.push(ip); }
                }
            }
        }
    }
    if ips.is_empty() { ips.push(std::net::Ipv4Addr::new(127, 0, 0, 1)); }
    ips
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredDevice {
    pub ip: String,
    pub mac: String,
    pub hostname: String,
    pub device_type: String,
    pub brand: String,
    pub description: String,
}

fn create_ping_command(ip_str: &str) -> tokio::process::Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = tokio::process::Command::new("ping");
        cmd.args(["-n", "1", "-w", "400", ip_str]);
        cmd.creation_flags(0x08000000);
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = tokio::process::Command::new("ping");
        cmd.args(["-c", "1", "-W", "1", ip_str]);
        cmd
    }
}

async fn ping_ip(ip: std::net::Ipv4Addr) {
    let ip_str = ip.to_string();
    let mut cmd = create_ping_command(&ip_str);
    let _ = cmd.output().await;
}

async fn sweep_subnet(local_ip: std::net::Ipv4Addr) {
    let octets = local_ip.octets();
    if octets[0] == 169 && octets[1] == 254 { return; }
    if octets[0] == 127 { return; }
    let semaphore = Arc::new(tokio::sync::Semaphore::new(64));
    let mut tasks = Vec::new();
    for i in 1..=254 {
        if i == octets[3] { continue; }
        let target_ip = std::net::Ipv4Addr::new(octets[0], octets[1], octets[2], i);
        let sem_clone = semaphore.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            ping_ip(target_ip).await;
        }));
    }
    for task in tasks { let _ = task.await; }
}

fn create_nslookup_command(ip: &str) -> tokio::process::Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = tokio::process::Command::new("nslookup");
        cmd.arg(ip);
        cmd.creation_flags(0x08000000);
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = tokio::process::Command::new("nslookup");
        cmd.arg(ip);
        cmd
    }
}

async fn resolve_hostname(ip: &str) -> Option<String> {
    let cmd_result = tokio::time::timeout(
        std::time::Duration::from_millis(400),
        create_nslookup_command(ip).output()
    ).await;
    let output = match cmd_result { Ok(Ok(o)) => o, _ => return None };
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("name:") || line_lower.contains("nombre:") || line_lower.contains("name =") {
            if let Some(pos) = line.find(':') {
                let name = line[pos + 1..].trim().trim_end_matches('.').to_string();
                if !name.is_empty() { return Some(name); }
            } else if let Some(pos) = line.find('=') {
                let name = line[pos + 1..].trim().trim_end_matches('.').to_string();
                if !name.is_empty() { return Some(name); }
            }
        }
    }
    None
}

fn parse_devices_from_arp_output(output: &str) -> Vec<(String, String)> {
    let mut devices = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 { continue; }
        let mut found_ip = None;
        let mut found_mac = None;
        for part in &parts {
            let clean_part = part.trim_matches(|c| c == '(' || c == ')');
            if let Ok(ip) = clean_part.parse::<std::net::Ipv4Addr>() {
                if is_valid_local_ip(&ip) { found_ip = Some(ip.to_string()); }
            } else if is_valid_mac_address(clean_part) {
                found_mac = Some(clean_part.replace('-', ":").to_lowercase());
            }
        }
        if let (Some(ip), Some(mac)) = (found_ip, found_mac) {
            if !devices.iter().any(|(existing_ip, _)| existing_ip == &ip) {
                devices.push((ip, mac));
            }
        }
    }
    devices
}

fn is_valid_mac_address(s: &str) -> bool {
    let clean = s.replace('-', ":").to_lowercase();
    if clean == "00:00:00:00:00:00" || clean == "ff:ff:ff:ff:ff:ff" { return false; }
    let parts: Vec<&str> = clean.split(':').collect();
    if parts.len() != 6 { return false; }
    for part in parts {
        if part.len() != 2 || !part.chars().all(|c| c.is_ascii_hexdigit()) { return false; }
    }
    true
}

fn get_brand_from_mac(mac: &str) -> (String, String) {
    let mac_clean = mac.replace('-', ":").to_lowercase();
    let prefix = if mac_clean.len() >= 8 { &mac_clean[0..8] } else { "" };
    match prefix {
        p if p.starts_with("b8:27:eb") || p.starts_with("d8:3a:dd") || p.starts_with("dc:a6:32") ||
             p.starts_with("e4:5f:01") || p.starts_with("28:cd:c1") || p.starts_with("d8:3a:dd") => {
            ("Raspberry Pi".to_string(), "pc".to_string())
        }
        p if p.starts_with("00:e0:4c") || p.starts_with("b8:27:eb") || p.starts_with("44:8a:5b") => {
            ("Realtek".to_string(), "pc".to_string())
        }
        p if p.starts_with("00:14:78") || p.starts_with("50:c7:bf") || p.starts_with("74:ea:3a") ||
             p.starts_with("ec:17:2f") => {
            ("TP-Link / Network Device".to_string(), "router".to_string())
        }
        _ => ("Dispositivo Genérico".to_string(), "unknown".to_string())
    }
}

fn guess_device_type_from_hostname(hostname: &str, default_type: &str) -> String {
    let h = hostname.to_lowercase();
    let phone_patterns = ["redmi","poco","xiaomi","miui","galaxy","samsung","moto","motorola",
        "iphone","android","phone","celular","movil","smartphone","huawei-p","oneplus","oppo","realme","vivo","pixel"];
    if phone_patterns.iter().any(|p| h.contains(p)) { return "mobile".to_string(); }
    let laptop_patterns = ["laptop","notebook","macbook","thinkpad","zenbook","vivobook",
        "inspiron","latitude","xps","pavilion","elitebook","probook","spectre","envy","yoga",
        "ideapad","legion","rog","zephyrus","tuf-","swift-","aspire","nitro-","predator","surface","chromebook"];
    if laptop_patterns.iter().any(|p| h.contains(p)) { return "laptop".to_string(); }
    let tv_patterns = ["smart-tv","smarttv","television","bravia","qled","roku","chromecast","firestick","androidtv","googletv","webos"];
    if tv_patterns.iter().any(|p| h.contains(p)) { return "tv".to_string(); }
    let printer_patterns = ["printer","impresora","epson","canon","brother","laserjet","deskjet","officejet","pixma","ecotank"];
    if printer_patterns.iter().any(|p| h.contains(p)) { return "printer".to_string(); }
    let router_patterns = ["router","gateway","modem","switch","tplink","tp-link","archer","deco","zte","tenda","d-link","netgear","linksys","mikrotik","ubiquiti","fritz","fritzbox"];
    if router_patterns.iter().any(|p| h.contains(p)) { return "router".to_string(); }
    let pc_patterns = ["desktop","workstation","computadora","torre","pc-","-pc","desktop-"];
    if pc_patterns.iter().any(|p| h.contains(p)) { return "pc".to_string(); }
    default_type.to_string()
}

fn infer_brand_from_hostname(hostname: &str) -> String {
    let h = hostname.to_lowercase();
    if h.contains("redmi") || h.contains("poco") || h.contains("xiaomi") { return "Xiaomi".to_string(); }
    if h.contains("samsung") || h.contains("galaxy") { return "Samsung".to_string(); }
    if h.contains("moto") || h.contains("motorola") { return "Motorola".to_string(); }
    if h.contains("iphone") || h.contains("ipad") || h.contains("macbook") { return "Apple".to_string(); }
    if h.contains("huawei") || h.contains("honor-") { return "Huawei".to_string(); }
    if h.contains("oneplus") { return "OnePlus".to_string(); }
    if h.contains("dell") { return "Dell".to_string(); }
    if h.contains("lenovo") || h.contains("thinkpad") || h.contains("ideapad") { return "Lenovo".to_string(); }
    if h.contains("hp") || h.contains("pavilion") { return "HP".to_string(); }
    if h.contains("acer") || h.contains("aspire") { return "Acer".to_string(); }
    if h.contains("rog") || h.contains("zenbook") || h.contains("asus") { return "ASUS".to_string(); }
    if h.contains("msi") { return "MSI".to_string(); }
    if h.contains("tplink") || h.contains("tp-link") { return "TP-Link".to_string(); }
    "Desconocido".to_string()
}

#[tauri::command]
async fn start_free_discovery() -> Result<Vec<DiscoveredDevice>, String> {
    let local_ips = get_all_local_ips();
    info!("Iniciando escaneo de red (ping sweep): {:?}", local_ips);
    let mut sweep_tasks = Vec::new();
    for ip in local_ips {
        sweep_tasks.push(tokio::spawn(async move { sweep_subnet(ip).await; }));
    }
    for task in sweep_tasks { let _ = task.await; }

    let mut arp_output = String::new();
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("arp").arg("-a").output() {
            arp_output = String::from_utf8_lossy(&output.stdout).to_string();
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/net/arp") {
            arp_output = content;
        } else if let Ok(output) = std::process::Command::new("arp").arg("-an").output() {
            arp_output = String::from_utf8_lossy(&output.stdout).to_string();
        }
    }

    let arp_devices = parse_devices_from_arp_output(&arp_output);
    info!("Parseados {} dispositivos desde tabla ARP.", arp_devices.len());

    let mut resolve_tasks = Vec::new();
    for (ip, mac) in arp_devices {
        resolve_tasks.push(tokio::spawn(async move {
            let hostname = resolve_hostname(&ip).await.unwrap_or_else(|| "unknown".to_string());
            let (mac_brand, default_type) = get_brand_from_mac(&mac);
            let device_type = guess_device_type_from_hostname(&hostname, &default_type);
            let brand = if mac_brand == "Dispositivo Genérico" { infer_brand_from_hostname(&hostname) } else { mac_brand };
            let last_octet = ip.split('.').last().unwrap_or("");
            let name = if hostname == "unknown" { format!("Dispositivo ({})", last_octet) } else {
                hostname.trim_end_matches(".local").trim_end_matches(".home").trim_end_matches(".lan").to_string()
            };
            let description = match (hostname == "unknown", brand == "Desconocido") {
                (true, _)  => format!("MAC: {} • Sin nombre resuelto", mac),
                (false, true)  => format!("IP: {}", ip),
                (false, false) => format!("{} • IP: {}", brand, ip),
            };
            DiscoveredDevice { ip, mac, hostname: name, device_type, brand, description }
        }));
    }

    let mut device_details = Vec::new();
    for task in resolve_tasks {
        if let Ok(device) = task.await { device_details.push(device); }
    }
    device_details.sort_by(|a, b| a.ip.cmp(&b.ip));
    Ok(device_details)
}

#[tauri::command]
async fn connect_to_peer(addr: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    info!("Conectando a {}...", addr);
    let clean_ip = addr.split(':').next().unwrap_or(&addr).to_string();
    {
        let conns = state.connections.lock().unwrap();
        if conns.keys().any(|k| k.starts_with(&clean_ip)) {
            info!("Ya existe una conexión activa con {}, omitiendo conexión duplicada", clean_ip);
            return Ok(format!("Ya conectado a {}", clean_ip));
        }
    }
    // Normalise address: if no port was given, use the server port 9876
    let server_addr = if addr.contains(':') { addr.clone() } else { format!("{}:9876", addr) };
    let conn = NetworkManager::connect(&server_addr, 9876).await?;
    // Store connection keyed by the normalized server address so disconnect works
    state.connections.lock().unwrap().insert(server_addr.clone(), conn.clone());
    start_receive_loop(conn, state.inner().clone(), server_addr.clone(), false);
    Ok(format!("Conectado a {}", server_addr))
}

#[tauri::command]
fn disconnect_from_peer(addr: String, state: tauri::State<AppState>) -> Result<(), String> {
    let clean_ip = addr.split(':').next().unwrap_or(&addr).to_string();
    // Find the actual key in the map (it may include port or not)
    let actual_key = {
        let conns = state.connections.lock().unwrap();
        conns.keys().find(|k| k.starts_with(&clean_ip)).cloned()
    };
    if let Some(key) = actual_key {
        let mut conns = state.connections.lock().unwrap();
        if let Some(conn) = conns.remove(&key) {
            conn.close(0u32.into(), b"Desconectado por el usuario");
            info!("Desconectado de {}", key);
        }
    }
    // Also remove by the original addr in case it was stored without prefix search
    state.remote_screens.lock().unwrap().retain(|k, _| !k.starts_with(&clean_ip));
    state.remote_audio_devices.lock().unwrap().retain(|k, _| !k.starts_with(&clean_ip));
    // Reset forwarding if we were forwarding to this peer
    let was_forwarding = {
        let fwd = state.forwarding_to.lock().unwrap();
        fwd.as_deref().map(|f| f.starts_with(&clean_ip)).unwrap_or(false)
    };
    if was_forwarding {
        *state.forwarding_to.lock().unwrap() = None;
        *state.cursor_on_remote.lock().unwrap() = false;
    }
    emit_connections_changed(&state);
    rebuild_virtual_layout(&state);
    Ok(())
}

#[tauri::command]
fn disconnect(state: tauri::State<AppState>) -> Result<(), String> {
    let mut conns = state.connections.lock().unwrap();
    for (addr, conn) in conns.drain() {
        conn.close(0u32.into(), b"Desconectado por el usuario");
        info!("Desconectado de {}", addr);
    }
    drop(conns);
    state.remote_screens.lock().unwrap().clear();
    state.remote_audio_devices.lock().unwrap().clear();
    *state.forwarding_to.lock().unwrap() = None;
    *state.cursor_on_remote.lock().unwrap() = false;
    emit_connections_changed(&state);
    rebuild_virtual_layout(&state);
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
    init_services(state.inner());
    Ok(())
}

#[tauri::command]
fn approve_connection(ip: String, always_allow: bool, state: tauri::State<AppState>) -> Result<(), String> {
    if always_allow {
        let mut config = state.config.lock().unwrap();
        if !config.connection.allowed_devices.contains(&ip) {
            config.connection.allowed_devices.push(ip.clone());
            config.save()?;
        }
    }
    if let Some(tx) = state.pending_approvals.lock().unwrap().remove(&ip) {
        let _ = tx.send(true);
    }
    info!("Conexión aprobada para {}", ip);
    Ok(())
}

#[tauri::command]
fn reject_connection(ip: String, state: tauri::State<AppState>) -> Result<(), String> {
    if let Some(tx) = state.pending_approvals.lock().unwrap().remove(&ip) {
        let _ = tx.send(false);
    }
    info!("Conexión rechazada para {}", ip);
    Ok(())
}

fn init_services(state: &AppState) {
    info!("Inicializando servicios según la configuración...");
    
    // 1. Detener servicios existentes primero
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

    let config = state.config.lock().unwrap().clone();

    // 2. Iniciar Portapapeles si está activo
    if config.services.clipboard_sync {
        let mut clipboard = ClipboardSync::new();
        match clipboard.init() {
            Ok(()) => {
                let state_clone = state.clone();
                clipboard.set_on_change(move |text| {
                    info!("Enviando portapapeles local a remoto...");
                    send_to_all_peers(NetworkMessage::Clipboard(text), &state_clone);
                });
                clipboard.start_monitoring();
                *state.clipboard.lock().unwrap() = Some(clipboard);
                info!("Servicio de portapapeles iniciado");
            }
            Err(e) => {
                error!("Error al iniciar portapapeles: {}", e);
            }
        }
    }

    // 3. Iniciar Ratón/Teclado si están activos
    if config.services.mouse_sharing || config.services.keyboard_sharing {
        let mut input = InputService::new();
        let state_clone = state.clone();
        input.set_on_input(move |event| {
            let is_forwarding = state_clone.forwarding_to.lock().unwrap().is_some();
            match &event {
                InputEvent::MouseMove { x, y } => {
                    handle_mouse_move(*x, *y, &state_clone);
                    if is_forwarding {
                        return;
                    }
                }
                _ => {}
            }

            if is_forwarding {
                let peer_ip = state_clone.forwarding_to.lock().unwrap().clone();
                if let Some(peer) = peer_ip {
                    let conn_addr = {
                        let conns = state_clone.connections.lock().unwrap();
                        conns.keys().find(|k| k.starts_with(&peer)).cloned()
                    };
                    if let Some(addr) = conn_addr {
                        let msg = match event {
                            InputEvent::MousePress { button } => Some(NetworkMessage::MouseEvent(network::MouseData {
                                event_type: network::MouseEventType::Press,
                                x: 0.0, y: 0.0,
                                button: Some(button),
                                scroll_delta: None,
                            })),
                            InputEvent::MouseRelease { button } => Some(NetworkMessage::MouseEvent(network::MouseData {
                                event_type: network::MouseEventType::Release,
                                x: 0.0, y: 0.0,
                                button: Some(button),
                                scroll_delta: None,
                            })),
                            InputEvent::MouseScroll { delta_x, delta_y } => Some(NetworkMessage::MouseEvent(network::MouseData {
                                event_type: network::MouseEventType::Scroll,
                                x: 0.0, y: 0.0,
                                button: None,
                                scroll_delta: Some((delta_x, delta_y)),
                            })),
                            InputEvent::KeyPress { key: _, char } => Some(NetworkMessage::KeyboardEvent(network::KeyboardData {
                                event_type: network::KeyboardEventType::Press,
                                key_code: 0,
                                key_char: char,
                            })),
                            InputEvent::KeyRelease { key: _, char } => Some(NetworkMessage::KeyboardEvent(network::KeyboardData {
                                event_type: network::KeyboardEventType::Release,
                                key_code: 0,
                                key_char: char,
                            })),
                            _ => None,
                        };
                        if let Some(m) = msg {
                            send_to_peer(&addr, m, &state_clone);
                        }
                    }
                }
            }
        });

        match input.start_capture() {
            Ok(()) => {
                *state.input_service.lock().unwrap() = Some(input);
                info!("Servicio de captura de entrada iniciado");
            }
            Err(e) => {
                error!("Error al iniciar captura de entrada: {}", e);
            }
        }
    }

    // 4. Iniciar Audio si está activo
    if config.services.audio_sharing {
        let source_route = config.audio.routes.iter().find(|r| r.source_pc == "local");
        let dest_route = config.audio.routes.iter().find(|r| r.dest_pc == "local");

        if let Some(route) = source_route {
            let dest_ip = route.dest_pc.clone();
            let source_device = route.source_device.clone();
            let dest_device = route.dest_device.clone();
            
            let state_clone = state.clone();
            let dest_ip_clone = dest_ip.clone();
            
            let mut audio = AudioService::new();
            audio.set_on_encoded_data(move |bytes| {
                let conn_addr = {
                    let conns = state_clone.connections.lock().unwrap();
                    conns.keys().find(|k| k.starts_with(&dest_ip_clone)).cloned()
                };
                if let Some(addr) = conn_addr {
                    send_to_peer(&addr, NetworkMessage::AudioData { route_id: "patchbay".to_string(), data: bytes }, &state_clone);
                }
            });

            if let Ok(()) = audio.start_capture(Some(&source_device)) {
                *state.audio_service.lock().unwrap() = Some(audio);
                info!("Servicio de audio (captura) auto-iniciado en {}", source_device);
                
                // Enviar comando para que el remoto inicie playback
                let conn_addr = {
                    let conns = state.connections.lock().unwrap();
                    conns.keys().find(|k| k.starts_with(&dest_ip)).cloned()
                };
                if let Some(addr) = conn_addr {
                    send_to_peer(&addr, NetworkMessage::StartAudioPlayback { device_name: dest_device }, state);
                }
            }
        }

        if let Some(route) = dest_route {
            let source_ip = route.source_pc.clone();
            let source_device = route.source_device.clone();
            let dest_device = route.dest_device.clone();

            let mut audio = AudioService::new();
            if let Ok(()) = audio.start_playback(Some(&dest_device)) {
                *state.audio_service.lock().unwrap() = Some(audio);
                info!("Servicio de audio (reproducción) auto-iniciado en {}", dest_device);

                // Enviar comando para que el remoto inicie captura
                let conn_addr = {
                    let conns = state.connections.lock().unwrap();
                    conns.keys().find(|k| k.starts_with(&source_ip)).cloned()
                };
                if let Some(addr) = conn_addr {
                    send_to_peer(&addr, NetworkMessage::StartAudioCapture { device_name: source_device }, state);
                }
            }
        }
    }
}

fn send_audio_devices(conn: &quinn::Connection) {
    let audio = AudioService::new();
    if let Ok((inputs, outputs)) = audio.list_devices() {
        let msg = NetworkMessage::AudioDevices { inputs, outputs };
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
}

#[tauri::command]
fn get_remote_audio_devices(state: tauri::State<AppState>) -> std::collections::HashMap<String, (Vec<audio::AudioDeviceInfo>, Vec<audio::AudioDeviceInfo>)> {
    state.remote_audio_devices.lock().unwrap().clone()
}

#[tauri::command]
fn refresh_audio_devices(state: tauri::State<AppState>) -> Result<(), String> {
    let conns = state.connections.lock().unwrap().clone();
    for (_, conn) in conns {
        send_audio_devices(&conn);
    }
    Ok(())
}

#[tauri::command]
async fn apply_audio_routes(routes: Vec<config::AudioRoute>, state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("Aplicando rutas de audio: {:?}", routes);
    
    // Guardar rutas en config
    {
        let mut config = state.config.lock().unwrap();
        config.audio.routes = routes.clone();
        let _ = config.save();
    }

    // Detener streams actuales por si acaso
    if let Some(ref mut audio) = *state.audio_service.lock().unwrap() {
        audio.stop();
    }

    // Buscar si hay una ruta donde somos el origen (captura)
    let source_route = routes.iter().find(|r| r.source_pc == "local");
    // Buscar si hay una ruta donde somos el destino (reproducción)
    let dest_route = routes.iter().find(|r| r.dest_pc == "local");

    // 1. Si somos origen de alguna ruta, iniciamos captura local y le decimos al destino remoto que empiece reproducción
    if let Some(route) = source_route {
        let dest_ip = route.dest_pc.clone();
        let source_device = route.source_device.clone();
        let dest_device = route.dest_device.clone();
        
        let state_clone = state.inner().clone();
        let dest_ip_clone = dest_ip.clone();
        
        // Inicializar servicio si es necesario
        let mut audio = AudioService::new();
        audio.set_on_encoded_data(move |bytes| {
            let conn_addr = {
                let conns = state_clone.connections.lock().unwrap();
                conns.keys().find(|k| k.starts_with(&dest_ip_clone)).cloned()
            };
            if let Some(addr) = conn_addr {
                send_to_peer(&addr, NetworkMessage::AudioData { route_id: "patchbay".to_string(), data: bytes }, &state_clone);
            }
        });

        info!("Iniciando captura local para ruta en: {}", source_device);
        audio.start_capture(Some(&source_device))?;
        *state.audio_service.lock().unwrap() = Some(audio);

        // Notificar al remoto que empiece a reproducir
        let conn_addr = {
            let conns = state.connections.lock().unwrap();
            conns.keys().find(|k| k.starts_with(&dest_ip)).cloned()
        };
        if let Some(addr) = conn_addr {
            send_to_peer(&addr, NetworkMessage::StartAudioPlayback { device_name: dest_device }, state.inner());
        }
    }

    // 2. Si somos destino de alguna ruta, iniciamos reproducción local y le decimos al origen remoto que empiece captura
    if let Some(route) = dest_route {
        let source_ip = route.source_pc.clone();
        let source_device = route.source_device.clone();
        let dest_device = route.dest_device.clone();

        let mut audio = AudioService::new();
        info!("Iniciando reproducción local en dispositivo: {}", dest_device);
        audio.start_playback(Some(&dest_device))?;
        *state.audio_service.lock().unwrap() = Some(audio);

        // Notificar al remoto que empiece a capturar
        let conn_addr = {
            let conns = state.connections.lock().unwrap();
            conns.keys().find(|k| k.starts_with(&source_ip)).cloned()
        };
        if let Some(addr) = conn_addr {
            send_to_peer(&addr, NetworkMessage::StartAudioCapture { device_name: source_device }, state.inner());
        }
    }

    // Si no somos ni origen ni destino de ninguna ruta activa, nos aseguramos de que el remoto detenga sus streams si estaban vinculados a nosotros
    if source_route.is_none() && dest_route.is_none() {
        // Enviar parada general a todos
        let conns = state.connections.lock().unwrap().clone();
        for addr in conns.keys() {
            send_to_peer(addr, NetworkMessage::StopAudioCapture, state.inner());
            send_to_peer(addr, NetworkMessage::StopAudioPlayback, state.inner());
        }
    }

    Ok(())
}

#[tauri::command]
fn start_clipboard_sync(state: tauri::State<AppState>) -> Result<(), String> {
    init_services(state.inner());
    Ok(())
}

#[tauri::command]
fn start_input_capture(state: tauri::State<AppState>) -> Result<(), String> {
    init_services(state.inner());
    Ok(())
}

#[tauri::command]
fn start_audio_capture(state: tauri::State<AppState>) -> Result<(), String> {
    init_services(state.inner());
    Ok(())
}

#[tauri::command]
fn list_audio_devices(_state: tauri::State<AppState>) -> Result<(Vec<audio::AudioDeviceInfo>, Vec<audio::AudioDeviceInfo>), String> {
    let audio = AudioService::new();
    audio.list_devices()
}

#[tauri::command]
fn stop_services(state: tauri::State<AppState>) -> Result<(), String> {
    if let Some(ref clipboard) = *state.clipboard.lock().unwrap() { clipboard.stop_monitoring(); }
    if let Some(ref input) = *state.input_service.lock().unwrap() { input.stop_capture(); }
    if let Some(ref mut audio) = *state.audio_service.lock().unwrap() { audio.stop(); }
    *state.clipboard.lock().unwrap() = None;
    *state.input_service.lock().unwrap() = None;
    *state.audio_service.lock().unwrap() = None;
    disconnect(state)?;
    Ok(())
}

#[tauri::command]
fn get_local_ips() -> serde_json::Value {
    let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "PC-Desconocido".to_string());
    let ips: Vec<String> = get_all_local_ips()
        .iter()
        .filter(|ip| !ip.is_loopback())
        .map(|ip| ip.to_string())
        .collect();
    serde_json::json!({ "hostname": hostname, "ips": ips })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PingResult {
    pub host: String,
    pub latency_ms: Option<f64>,
    pub success: bool,
    pub error: Option<String>,
}

#[tauri::command]
async fn ping_host(host: String) -> Result<PingResult, String> {
    let target = if host.is_empty() { "8.8.8.8".to_string() } else { host.clone() };
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = tokio::process::Command::new("ping");
        c.args(["-n", "1", "-w", "2000", &target]);
        c.creation_flags(0x08000000);
        c
    };
    #[cfg(not(target_os = "windows"))]
    let mut cmd = {
        let mut c = tokio::process::Command::new("ping");
        c.args(["-c", "1", "-W", "2", &target]);
        c
    };
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), cmd.output()).await;
    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
            let latency = parse_ping_latency(&stdout);
            Ok(PingResult {
                host: target,
                latency_ms: latency,
                success: latency.is_some(),
                error: if latency.is_none() { Some("Sin respuesta".to_string()) } else { None },
            })
        }
        Ok(Err(e)) => Ok(PingResult { host: target, latency_ms: None, success: false, error: Some(e.to_string()) }),
        Err(_) => Ok(PingResult { host: target, latency_ms: None, success: false, error: Some("Timeout".to_string()) }),
    }
}

fn parse_ping_latency(output: &str) -> Option<f64> {
    for line in output.lines() {
        if let Some(pos) = line.find("tiempo=").or_else(|| line.find("time=")) {
            let rest = &line[pos..];
            let after = rest.splitn(2, '=').nth(1)?;
            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
            if let Ok(ms) = num_str.parse::<f64>() { return Some(ms); }
        }
        if line.contains("time=") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in &parts {
                if part.starts_with("time=") {
                    let num_str: String = part[5..].chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
                    if let Ok(ms) = num_str.parse::<f64>() { return Some(ms); }
                }
            }
        }
    }
    None
}

// ===== SCREEN LAYOUT HELPERS =====

fn collect_local_screens() -> Vec<ScreenInfo> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command",
                "Add-Type -AssemblyName System.Windows.Forms; \
                [System.Windows.Forms.Screen]::AllScreens | ForEach-Object { \
                    $b = $_.Bounds; \
                    Write-Output \"$($b.X),$($b.Y),$($b.Width),$($b.Height),$($_.Primary),$($_.DeviceName)\" \
                }"
            ])
            .output();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut screens = Vec::new();
            for (i, line) in text.lines().enumerate() {
                let parts: Vec<&str> = line.trim().split(',').collect();
                if parts.len() >= 5 {
                    let x: i32 = parts[0].parse().unwrap_or(0);
                    let y: i32 = parts[1].parse().unwrap_or(0);
                    let w: u32 = parts[2].parse().unwrap_or(1920);
                    let h: u32 = parts[3].parse().unwrap_or(1080);
                    let primary = parts[4].trim().eq_ignore_ascii_case("true");
                    let name = if parts.len() >= 6 { parts[5..].join(",").trim().to_string() } else { format!("Display {}", i + 1) };
                    screens.push(ScreenInfo {
                        id: format!("screen-{}", i),
                        name: name.trim_start_matches("\\\\.\\DISPLAY").to_string(),
                        x, y, width: w, height: h, is_primary: primary,
                    });
                }
            }
            if !screens.is_empty() { return screens; }
        }
        vec![ScreenInfo { id: "screen-0".to_string(), name: "Pantalla principal".to_string(), x: 0, y: 0, width: 1920, height: 1080, is_primary: true }]
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let output = Command::new("xrandr").arg("--query").output();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut screens = Vec::new();
            let mut idx = 0usize;
            for line in text.lines() {
                if line.contains(" connected ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let name = parts[0].to_string();
                    for part in &parts {
                        if part.contains('x') && part.contains('+') {
                            let geom: Vec<&str> = part.split(|c| c == 'x' || c == '+').collect();
                            if geom.len() >= 4 {
                                let w: u32 = geom[0].parse().unwrap_or(1920);
                                let h: u32 = geom[1].parse().unwrap_or(1080);
                                let x: i32 = geom[2].parse().unwrap_or(0);
                                let y: i32 = geom[3].parse().unwrap_or(0);
                                screens.push(ScreenInfo { id: format!("screen-{}", idx), name: name.clone(), x, y, width: w, height: h, is_primary: idx == 0 });
                                idx += 1;
                                break;
                            }
                        }
                    }
                }
            }
            if !screens.is_empty() { return screens; }
        }
        vec![ScreenInfo { id: "screen-0".to_string(), name: "Pantalla principal".to_string(), x: 0, y: 0, width: 1920, height: 1080, is_primary: true }]
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        vec![ScreenInfo { id: "screen-0".to_string(), name: "Pantalla principal".to_string(), x: 0, y: 0, width: 1920, height: 1080, is_primary: true }]
    }
}

// ===== DEVICE LINKING COMMANDS =====

#[tauri::command]
fn get_local_screens() -> Vec<ScreenInfo> {
    collect_local_screens()
}

#[tauri::command]
fn get_remote_screens(state: tauri::State<AppState>) -> std::collections::HashMap<String, Vec<ScreenInfo>> {
    state.remote_screens.lock().unwrap().clone()
}

#[tauri::command]
fn get_virtual_layout(state: tauri::State<AppState>) -> Vec<VirtualScreen> {
    state.virtual_layout.lock().unwrap().clone()
}

/// Called from frontend when the user manually repositions screens in the canvas.
/// Also propagates the new layout to all connected peers so their crossover logic is updated.
#[tauri::command]
fn set_virtual_layout(layout: Vec<VirtualScreen>, state: tauri::State<AppState>) -> Result<(), String> {
    info!("Layout virtual actualizado con {} pantallas", layout.len());
    *state.virtual_layout.lock().unwrap() = layout;
    // Send updated local screen layout to all peers so they can adjust their crossover zones
    let local_screens = collect_local_screens();
    send_to_all_peers(NetworkMessage::ScreenLayout(local_screens), state.inner());
    Ok(())
}

#[tauri::command]
fn link_device(ip: String, name: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    if !config.linked_devices.iter().any(|d| d.ip == ip) {
        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        config.linked_devices.push(LinkedDevice { ip: ip.clone(), name: name.clone(), linked_at: ts });
        config.save()?;
        info!("Dispositivo vinculado: {} ({})", name, ip);
    }
    Ok(())
}

#[tauri::command]
fn unlink_device(ip: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.linked_devices.retain(|d| d.ip != ip);
    config.save()?;
    info!("Dispositivo desvinculado: {}", ip);
    Ok(())
}

#[tauri::command]
fn get_linked_devices(state: tauri::State<AppState>) -> Vec<LinkedDevice> {
    state.config.lock().unwrap().linked_devices.clone()
}

#[tauri::command]
fn get_cursor_on_remote(state: tauri::State<AppState>) -> bool {
    *state.cursor_on_remote.lock().unwrap()
}

#[tauri::command]
fn set_cursor_on_remote(value: bool, state: tauri::State<AppState>) {
    *state.cursor_on_remote.lock().unwrap() = value;
    if !value {
        *state.forwarding_to.lock().unwrap() = None;
    }
}

#[tauri::command]
async fn toggle_wifi_hotspot(enable: bool) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let action = if enable { "StartTetheringAsync" } else { "StopTetheringAsync" };
        let cmd = format!(
            "$profile = Get-NetConnectionProfile | Where-Object {{ $_.IPv4Connectivity -eq 'Internet' }} | Select-Object -First 1; \
             if (-not $profile) {{ $profile = Get-NetConnectionProfile | Select-Object -First 1 }}; \
             if (-not $profile) {{ throw 'No hay interfaz de red activa para compartir.' }}; \
             $manager = [Windows.Networking.NetworkOperators.NetworkOperatorTetheringManager, Windows.Networking.NetworkOperators, ContentType = WindowsRuntime]::CreateFromConnectionProfile($profile); \
             $asyncOp = $manager.{}(); \
             $asyncOp.GetResults()",
            action
        );
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &cmd])
            .output();
        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(if enable { "Punto de acceso iniciado con éxito." } else { "Punto de acceso detenido." }.to_string())
                } else {
                    let err = String::from_utf8_lossy(&out.stderr).to_string();
                    Err(format!("Error de configuración: {}", err))
                }
            }
            Err(e) => Err(e.to_string())
        }
    }
    #[cfg(target_os = "linux")]
    {
        if enable {
            let output = std::process::Command::new("nmcli")
                .args(["device", "wifi", "hotspot", "ssid", "NetBridgeHotspot", "password", "netbridge1234"])
                .output();
            match output {
                Ok(out) => {
                    if out.status.success() {
                        Ok("Punto de acceso creado (NetBridgeHotspot / netbridge1234)".to_string())
                    } else {
                        Err(String::from_utf8_lossy(&out.stderr).to_string())
                    }
                }
                Err(e) => Err(e.to_string())
            }
        } else {
            let _ = std::process::Command::new("nmcli").args(["connection", "down", "Hotspot"]).status();
            Ok("Punto de acceso desactivado".to_string())
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("No soportado en este sistema operativo".to_string())
    }
}

#[tauri::command]
async fn get_wifi_hotspot_status() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let cmd = " \
             $profile = Get-NetConnectionProfile | Where-Object { $_.IPv4Connectivity -eq 'Internet' } | Select-Object -First 1; \
             if (-not $profile) { $profile = Get-NetConnectionProfile | Select-Object -First 1 }; \
             if (-not $profile) { Write-Output 'Off'; exit }; \
             $manager = [Windows.Networking.NetworkOperators.NetworkOperatorTetheringManager, Windows.Networking.NetworkOperators, ContentType = WindowsRuntime]::CreateFromConnectionProfile($profile); \
             Write-Output $manager.TetheringOperationalState";
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", cmd])
            .output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                Ok(stdout)
            }
            Err(e) => Err(e.to_string())
        }
    }
    #[cfg(target_os = "linux")]
    {
        let output = std::process::Command::new("nmcli")
            .args(["-t", "-f", "NAME,ACTIVE", "connection", "show", "--active"])
            .output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.contains("Hotspot") {
                    Ok("On".to_string())
                } else {
                    Ok("Off".to_string())
                }
            }
            Err(e) => Err(e.to_string())
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Ok("Off".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    configure_system_firewall();
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .setup(|app| {
            let state = app.state::<AppState>();
            *state.app_handle.lock().unwrap() = Some(app.handle().clone());
            let state_clone = state.inner().clone();
            
            // Iniciar descubrimiento en segundo plano
            let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "PC-Desconocido".to_string());
            let mut discovery = DiscoveryService::new();
            match discovery.start(&hostname, 9876) {
                Ok(()) => {
                    *state.discovery.lock().unwrap() = Some(discovery);
                    info!("Servicio de descubrimiento mDNS+UDP iniciado para {}", hostname);
                }
                Err(e) => { error!("Error al iniciar servicio de descubrimiento: {}", e); }
            }

            // Build initial virtual layout from local screens
            rebuild_virtual_layout(&state);

            // Inicializar servicios KVM (mouse, portapapeles, etc.) según la configuración
            init_services(&state);

            // Iniciar servidor QUIC en puerto 9876
            tauri::async_runtime::spawn(async move {
                let mut network_mgr = NetworkManager::new(state_clone.config.lock().unwrap().clone());
                let endpoint = match network_mgr.start_server(9876).await {
                    Ok(ep) => ep,
                    Err(e) => { error!("Error al iniciar servidor QUIC: {}", e); return; }
                };
                *state_clone.network.lock().unwrap() = Some(network_mgr);
                info!("Servidor QUIC iniciado en puerto 9876. Esperando conexiones...");
                loop {
                    match endpoint.accept().await {
                        Some(incoming) => {
                            let state_nested = state_clone.clone();
                            tauri::async_runtime::spawn(async move {
                                match incoming.await {
                                    Ok(conn) => {
                                        let remote_addr = conn.remote_address().to_string();
                                        info!("Conexión entrante aceptada de {}", remote_addr);
                                        state_nested.connections.lock().unwrap().insert(remote_addr.clone(), conn.clone());
                                        start_receive_loop(conn, state_nested, remote_addr, true);
                                    }
                                    Err(e) => { error!("Error al aceptar conexión QUIC entrante: {}", e); }
                                }
                            });
                        }
                        None => { error!("Endpoint QUIC cerrado"); break; }
                    }
                }
            });
            
            // Auto-connect to linked devices
            let auto_connect = state.config.lock().unwrap().general.auto_connect;
            if auto_connect {
                let linked = state.config.lock().unwrap().linked_devices.clone();
                let state_auto = state.inner().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    for device in linked {
                        // Normalize address to include port
                        let addr_with_port = if device.ip.contains(':') { device.ip.clone() } else { format!("{}:9876", device.ip) };
                        info!("Auto-conectando a: {} ({}) => {}", device.name, device.ip, addr_with_port);
                        match NetworkManager::connect(&addr_with_port, 9876).await {
                            Ok(conn) => {
                                state_auto.connections.lock().unwrap().insert(addr_with_port.clone(), conn.clone());
                                start_receive_loop(conn, state_auto.clone(), addr_with_port.clone(), false);
                                info!("Auto-conexión exitosa a {}", addr_with_port);
                            }
                            Err(e) => { warn!("No se pudo auto-conectar a {}: {}", addr_with_port, e); }
                        }
                    }
                });
            }

            // ── System Tray ──────────────────────────────────────────────
            let show_item = tauri::menu::MenuItem::with_id(app, "show", "Mostrar NetBridge", true, None::<&str>)?;
            let quit_item = tauri::menu::MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
            let tray_menu = tauri::menu::Menu::with_items(app, &[&show_item, &quit_item])?;
            let _tray = TrayIconBuilder::new()
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .icon(app.default_window_icon().cloned().unwrap())
                .tooltip("NetBridge")
                .on_menu_event(|app_handle, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(win) = app_handle.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                        "quit" => {
                            info!("Saliendo de NetBridge desde la bandeja");
                            app_handle.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::DoubleClick { button: tauri::tray::MouseButton::Left, .. } = event {
                        if let Some(win) = tray.app_handle().get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;
            // ─────────────────────────────────────────────────────────────
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let minimize = {
                    // Access app state to check minimize_to_tray setting
                    if let Some(state) = window.try_state::<AppState>() {
                        state.config.lock().unwrap().general.minimize_to_tray
                    } else {
                        false
                    }
                };
                if minimize {
                    api.prevent_close();
                    let _ = window.hide();
                    info!("Ventana minimizada a bandeja del sistema");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            start_discovery,
            start_free_discovery,
            get_discovered_peers,
            connect_to_peer,
            disconnect_from_peer,
            disconnect,
            get_connection_status,
            get_connected_peers,
            get_config,
            update_config,
            approve_connection,
            reject_connection,
            start_clipboard_sync,
            start_input_capture,
            start_audio_capture,
            list_audio_devices,
            stop_services,
            get_local_ips,
            ping_host,
            get_local_screens,
            get_remote_screens,
            get_virtual_layout,
            set_virtual_layout,
            link_device,
            unlink_device,
            get_linked_devices,
            get_cursor_on_remote,
            set_cursor_on_remote,
            toggle_wifi_hotspot,
            get_wifi_hotspot_status,
            apply_audio_routes,
            get_remote_audio_devices,
            refresh_audio_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
