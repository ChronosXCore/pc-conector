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



fn get_local_ip_via_udp() -> Option<std::net::Ipv4Addr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    match socket.local_addr().ok()?.ip() {
        std::net::IpAddr::V4(ip) => Some(ip),
        _ => None,
    }
}

fn get_all_local_ips() -> Vec<std::net::Ipv4Addr> {
    let mut ips = Vec::new();
    
    if let Some(ip) = get_local_ip_via_udp() {
        ips.push(ip);
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("ipconfig").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("IPv4") {
                    if let Some(pos) = line.rfind(':') {
                        let ip_str = line[pos + 1..].trim();
                        if let Ok(ip) = ip_str.parse::<std::net::Ipv4Addr>() {
                            if !ip.is_loopback() && !ips.contains(&ip) {
                                ips.push(ip);
                            }
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(output) = std::process::Command::new("ip").args(["addr", "show"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("inet ") && !line.contains("127.0.0.1") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    for part in parts {
                        if part.contains('/') {
                            if let Some(ip_part) = part.split('/').next() {
                                if let Ok(ip) = ip_part.parse::<std::net::Ipv4Addr>() {
                                    if !ips.contains(&ip) {
                                        ips.push(ip);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    if ips.is_empty() {
        ips.push(std::net::Ipv4Addr::new(127, 0, 0, 1));
    }
    
    ips
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredDevice {
    pub ip: String,
    pub mac: String,
    pub hostname: String,
    pub device_type: String, // "pc", "laptop", "mobile", "router", "tv", "printer", "unknown"
    pub brand: String,
    pub description: String,
}

fn create_ping_command(ip_str: &str) -> tokio::process::Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = tokio::process::Command::new("ping");
        cmd.args(["-n", "1", "-w", "400", ip_str]);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
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
    if octets[0] == 169 && octets[1] == 254 {
        return; // Skip link-local autoconfiguration (APIPA)
    }
    if octets[0] == 127 {
        return; // Skip loopback
    }

    let semaphore = Arc::new(tokio::sync::Semaphore::new(64));
    let mut tasks = Vec::new();

    for i in 1..=254 {
        if i == octets[3] {
            continue;
        }
        let target_ip = std::net::Ipv4Addr::new(octets[0], octets[1], octets[2], i);
        let sem_clone = semaphore.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            ping_ip(target_ip).await;
        }));
    }

    for task in tasks {
        let _ = task.await;
    }
}

fn create_nslookup_command(ip: &str) -> tokio::process::Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = tokio::process::Command::new("nslookup");
        cmd.arg(ip);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
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
    
    let output = match cmd_result {
        Ok(Ok(o)) => o,
        _ => return None,
    };
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("name:") || line_lower.contains("nombre:") || line_lower.contains("name =") {
            if let Some(pos) = line.find(':') {
                let name = line[pos + 1..].trim().trim_end_matches('.').to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            } else if let Some(pos) = line.find('=') {
                let name = line[pos + 1..].trim().trim_end_matches('.').to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }
    None
}

fn parse_devices_from_arp_output(output: &str) -> Vec<(String, String)> {
    let mut devices = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        
        let mut found_ip = None;
        let mut found_mac = None;
        
        for part in &parts {
            let clean_part = part.trim_matches(|c| c == '(' || c == ')');
            if let Ok(ip) = clean_part.parse::<std::net::Ipv4Addr>() {
                if is_valid_local_ip(&ip) {
                    found_ip = Some(ip.to_string());
                }
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
    let parts: Vec<&str> = clean.split(':').collect();
    if parts.len() != 6 {
        return false;
    }
    for part in parts {
        if part.len() != 2 || !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
    }
    true
}

fn get_brand_from_mac(mac: &str) -> (String, String) {
    let mac_clean = mac.replace('-', ":").to_lowercase();
    let prefix = if mac_clean.len() >= 8 {
        &mac_clean[0..8]
    } else {
        ""
    };
    
    match prefix {
        p if p.starts_with("00:1c:b3") || p.starts_with("00:25:00") || p.starts_with("00:26:bb") ||
             p.starts_with("3c:d0:f8") || p.starts_with("f0:18:98") || p.starts_with("f8:27:93") ||
             p.starts_with("00:03:93") || p.starts_with("00:05:02") || p.starts_with("00:0a:27") ||
             p.starts_with("00:0d:93") || p.starts_with("00:10:fa") || p.starts_with("00:1e:c2") ||
             p.starts_with("00:23:32") || p.starts_with("00:25:4b") || p.starts_with("00:30:65") ||
             p.starts_with("04:0c:ce") || p.starts_with("04:15:52") || p.starts_with("04:1e:64") ||
             p.starts_with("04:26:65") || p.starts_with("04:54:53") || p.starts_with("04:e5:36") ||
             p.starts_with("0c:51:01") || p.starts_with("0c:74:c2") || p.starts_with("10:1c:0c") ||
             p.starts_with("10:40:f3") || p.starts_with("10:93:e9") || p.starts_with("10:dd:b1") ||
             p.starts_with("14:10:9f") || p.starts_with("14:20:5e") || p.starts_with("14:5a:05") ||
             p.starts_with("14:99:e2") || p.starts_with("1c:1a:c0") || p.starts_with("1c:ab:a7") ||
             p.starts_with("20:3c:ae") || p.starts_with("24:a2:e1") || p.starts_with("2c:20:0b") ||
             p.starts_with("2c:be:08") || p.starts_with("2c:f0:ee") || p.starts_with("30:07:4d") ||
             p.starts_with("30:57:14") || p.starts_with("34:08:bc") || p.starts_with("34:15:9e") ||
             p.starts_with("34:36:3b") || p.starts_with("34:a8:eb") || p.starts_with("38:ca:da") ||
             p.starts_with("3c:07:54") || p.starts_with("3c:15:c2") || p.starts_with("3c:37:86") ||
             p.starts_with("3c:a6:16") || p.starts_with("40:30:04") || p.starts_with("40:3c:fc") ||
             p.starts_with("40:4d:7f") || p.starts_with("40:9c:28") || p.starts_with("44:2a:60") ||
             p.starts_with("44:d8:84") || p.starts_with("48:43:7c") || p.starts_with("48:d7:05") ||
             p.starts_with("4c:32:75") || p.starts_with("4c:74:bf") || p.starts_with("4c:b1:cd") ||
             p.starts_with("50:23:00") || p.starts_with("50:bc:96") || p.starts_with("54:26:96") ||
             p.starts_with("54:33:cb") || p.starts_with("54:99:63") || p.starts_with("54:ae:27") ||
             p.starts_with("58:11:22") || p.starts_with("58:40:3e") || p.starts_with("58:55:ca") ||
             p.starts_with("58:e6:ba") || p.starts_with("5c:59:48") || p.starts_with("5c:8d:4e") ||
             p.starts_with("5c:95:ae") || p.starts_with("5c:f9:dd") || p.starts_with("60:03:08") ||
             p.starts_with("60:30:d4") || p.starts_with("60:a3:75") || p.starts_with("60:c5:47") ||
             p.starts_with("60:d0:a9") || p.starts_with("60:f8:1d") || p.starts_with("64:20:0c") ||
             p.starts_with("64:70:39") || p.starts_with("64:9a:11") || p.starts_with("64:b9:e8") ||
             p.starts_with("64:c6:af") || p.starts_with("64:e6:82") || p.starts_with("68:5b:35") ||
             p.starts_with("68:ae:20") || p.starts_with("68:d9:3c") || p.starts_with("6c:19:c0") ||
             p.starts_with("6c:3e:6d") || p.starts_with("6c:40:08") || p.starts_with("6c:70:9f") ||
             p.starts_with("6c:8d:c1") || p.starts_with("6c:96:cf") || p.starts_with("6c:c2:6b") ||
             p.starts_with("70:11:24") || p.starts_with("70:3e:ac") || p.starts_with("70:81:eb") ||
             p.starts_with("70:a2:b3") || p.starts_with("70:cd:60") || p.starts_with("70:de:e2") ||
             p.starts_with("74:1b:b2") || p.starts_with("74:81:14") || p.starts_with("74:8d:08") ||
             p.starts_with("74:e1:b6") || p.starts_with("74:f6:12") || p.starts_with("78:31:c1") ||
             p.starts_with("78:4f:43") || p.starts_with("78:7b:8a") || p.starts_with("78:88:6d") ||
             p.starts_with("78:ca:39") || p.starts_with("78:fd:94") || p.starts_with("7c:04:d0") ||
             p.starts_with("7c:11:be") || p.starts_with("7c:50:49") || p.starts_with("7c:6d:62") ||
             p.starts_with("7c:d1:c3") || p.starts_with("80:00:6e") || p.starts_with("80:49:71") ||
             p.starts_with("80:92:9f") || p.starts_with("80:b0:3d") || p.starts_with("80:ea:96") ||
             p.starts_with("84:29:99") || p.starts_with("84:38:35") || p.starts_with("84:78:8b") ||
             p.starts_with("84:8e:0c") || p.starts_with("84:b1:53") || p.starts_with("84:fc:fe") ||
             p.starts_with("88:19:08") || p.starts_with("88:53:95") || p.starts_with("88:63:df") ||
             p.starts_with("88:c6:63") || p.starts_with("88:cb:87") || p.starts_with("8c:29:37") ||
             p.starts_with("8c:2d:aa") || p.starts_with("8c:58:77") || p.starts_with("8c:7a:3d") ||
             p.starts_with("8c:85:90") || p.starts_with("8c:fa:ba") || p.starts_with("90:27:e4") ||
             p.starts_with("90:3c:ab") || p.starts_with("90:72:40") || p.starts_with("90:84:0d") ||
             p.starts_with("90:b1:1c") || p.starts_with("90:e2:cf") || p.starts_with("90:f0:52") ||
             p.starts_with("94:10:3f") || p.starts_with("94:16:25") || p.starts_with("94:7b:e7") ||
             p.starts_with("94:94:26") || p.starts_with("94:b4:0f") || p.starts_with("94:e9:64") ||
             p.starts_with("94:f6:d6") || p.starts_with("98:01:a7") || p.starts_with("98:10:e8") ||
             p.starts_with("98:5a:eb") || p.starts_with("98:9e:63") || p.starts_with("98:b6:e9") ||
             p.starts_with("98:d6:bb") || p.starts_with("98:f0:ab") || p.starts_with("9c:04:eb") ||
             p.starts_with("9c:20:7b") || p.starts_with("9c:35:eb") || p.starts_with("9c:4f:da") ||
             p.starts_with("9c:8b:c0") || p.starts_with("9c:f3:87") || p.starts_with("a0:18:28") ||
             p.starts_with("a0:3b:8f") || p.starts_with("a0:99:9b") || p.starts_with("a0:ed:cd") ||
             p.starts_with("a4:31:35") || p.starts_with("a4:5e:60") || p.starts_with("a4:b1:97") ||
             p.starts_with("a4:c3:61") || p.starts_with("a4:d1:8c") || p.starts_with("a4:e9:75") ||
             p.starts_with("a4:f1:e8") || p.starts_with("a8:20:66") || p.starts_with("a8:5b:78") ||
             p.starts_with("a8:60:b6") || p.starts_with("a8:66:7f") || p.starts_with("a8:88:08") ||
             p.starts_with("a8:8e:24") || p.starts_with("a8:bb:cf") || p.starts_with("ac:1f:74") ||
             p.starts_with("ac:29:3a") || p.starts_with("ac:3c:0b") || p.starts_with("ac:7f:3e") ||
             p.starts_with("ac:bc:32") || p.starts_with("ac:cf:85") || p.starts_with("ac:ec:80") ||
             p.starts_with("b0:19:c6") || p.starts_with("b0:34:95") || p.starts_with("b0:65:bd") ||
             p.starts_with("b0:70:2d") || p.starts_with("b0:9f:ba") || p.starts_with("b0:ca:68") ||
             p.starts_with("b4:18:d1") || p.starts_with("b4:8b:19") || p.starts_with("b4:f0:5a") ||
             p.starts_with("b4:f6:1c") || p.starts_with("b8:09:8a") || p.starts_with("b8:53:ac") ||
             p.starts_with("b8:63:bc") || p.starts_with("b8:78:f3") || p.starts_with("b8:8d:12") ||
             p.starts_with("b8:c7:5d") || p.starts_with("b8:e8:56") || p.starts_with("b8:f6:b1") ||
             p.starts_with("bc:3b:af") || p.starts_with("bc:4c:c4") || p.starts_with("bc:52:b7") ||
             p.starts_with("bc:67:78") || p.starts_with("bc:9f:ef") || p.starts_with("bc:ec:5d") ||
             p.starts_with("c0:1a:da") || p.starts_with("c0:38:96") || p.starts_with("c0:84:7a") ||
             p.starts_with("c0:9f:42") || p.starts_with("c0:cc:f8") || p.starts_with("c0:d0:12") ||
             p.starts_with("c0:ee:fb") || p.starts_with("c0:f2:fb") || p.starts_with("c4:2c:03") ||
             p.starts_with("c4:98:80") || p.starts_with("c4:b3:01") || p.starts_with("c4:d9:87") ||
             p.starts_with("c4:e9:84") || p.starts_with("c8:1e:e7") || p.starts_with("c8:2a:14") ||
             p.starts_with("c8:33:4b") || p.starts_with("c8:69:cd") || p.starts_with("c8:85:50") ||
             p.starts_with("c8:b5:b7") || p.starts_with("c8:bc:c8") || p.starts_with("c8:d0:83") ||
             p.starts_with("c8:e0:eb") || p.starts_with("c8:f6:50") || p.starts_with("cc:08:e0") ||
             p.starts_with("cc:20:e8") || p.starts_with("cc:25:ef") || p.starts_with("cc:29:f5") ||
             p.starts_with("cc:78:5f") || p.starts_with("cc:c7:60") || p.starts_with("d0:03:4b") ||
             p.starts_with("d0:22:be") || p.starts_with("d0:23:db") || p.starts_with("d0:25:99") ||
             p.starts_with("d0:33:11") || p.starts_with("d0:4f:7e") || p.starts_with("d0:81:7a") ||
             p.starts_with("d0:a6:37") || p.starts_with("d0:c5:f3") || p.starts_with("d0:d2:b0") ||
             p.starts_with("d0:e1:40") || p.starts_with("d4:28:d5") || p.starts_with("d4:3a:2c") ||
             p.starts_with("d4:61:9d") || p.starts_with("d4:90:9c") || p.starts_with("d4:a3:3d") ||
             p.starts_with("d4:dc:cd") || p.starts_with("d4:f4:6f") || p.starts_with("d8:00:4d") ||
             p.starts_with("d8:1c:79") || p.starts_with("d8:30:62") || p.starts_with("d8:8f:76") ||
             p.starts_with("d8:96:95") || p.starts_with("d8:a2:5e") || p.starts_with("d8:bb:2c") ||
             p.starts_with("d8:c7:71") || p.starts_with("d8:d1:cb") || p.starts_with("d8:e0:e1") ||
             p.starts_with("dc:0c:5c") || p.starts_with("dc:2b:2a") || p.starts_with("dc:37:14") ||
             p.starts_with("dc:41:5f") || p.starts_with("dc:86:d8") || p.starts_with("dc:a9:04") ||
             p.starts_with("dc:d3:a2") || p.starts_with("dc:e4:6b") || p.starts_with("e0:2a:b3") ||
             p.starts_with("e0:5b:d4") || p.starts_with("e0:66:78") || p.starts_with("e0:ac:cb") ||
             p.starts_with("e0:b5:2d") || p.starts_with("e0:b9:ba") || p.starts_with("e0:c9:7a") ||
             p.starts_with("e0:db:55") || p.starts_with("e0:f5:c6") || p.starts_with("e0:f8:47") ||
             p.starts_with("e4:25:e9") || p.starts_with("e4:50:eb") || p.starts_with("e4:8b:7f") ||
             p.starts_with("e4:9a:dc") || p.starts_with("e4:b2:fb") || p.starts_with("e4:c7:22") ||
             p.starts_with("e4:e0:a6") || p.starts_with("e4:e4:ab") || p.starts_with("e8:04:0b") ||
             p.starts_with("e8:06:88") || p.starts_with("e8:1b:5b") || p.starts_with("e8:80:2e") ||
             p.starts_with("e8:8d:28") || p.starts_with("e8:b2:ac") || p.starts_with("e8:e9:a4") ||
             p.starts_with("ec:08:6b") || p.starts_with("ec:1d:7b") || p.starts_with("ec:2c:e2") ||
             p.starts_with("ec:35:86") || p.starts_with("ec:85:2f") || p.starts_with("ec:ad:b8") ||
             p.starts_with("f0:24:75") || p.starts_with("f0:76:6f") || p.starts_with("f0:99:bf") ||
             p.starts_with("f0:b4:79") || p.starts_with("f0:c1:f1") || p.starts_with("f0:db:f8") ||
             p.starts_with("f0:f7:55") || p.starts_with("f4:0f:24") || p.starts_with("f4:1b:a1") ||
             p.starts_with("f4:37:b7") || p.starts_with("f4:5c:89") || p.starts_with("f4:f9:51") ||
             p.starts_with("f4:f5:d8") || p.starts_with("f8:03:77") || p.starts_with("f8:1e:df") ||
             p.starts_with("f8:38:80") || p.starts_with("f8:62:14") || p.starts_with("f8:6f:c1") ||
             p.starts_with("f8:87:f1") || p.starts_with("f8:e9:03") || p.starts_with("fc:18:07") ||
             p.starts_with("fc:25:3f") || p.starts_with("fc:2a:54") || p.starts_with("fc:2d:c4") ||
             p.starts_with("fc:d8:12") || p.starts_with("fc:e9:98") || p.starts_with("fc:fc:48") => {
                 ("Apple".to_string(), "mobile".to_string())
             }
        
        p if p.starts_with("00:00:f0") || p.starts_with("00:07:ab") || p.starts_with("00:12:47") ||
             p.starts_with("18:22:7f") || p.starts_with("1c:5a:3e") || p.starts_with("38:ec:11") ||
             p.starts_with("48:5a:3f") || p.starts_with("50:b7:c3") || p.starts_with("8c:c8:cd") ||
             p.starts_with("94:e1:ac") || p.starts_with("d8:e0:e1") || p.starts_with("f4:e3:fb") ||
             p.starts_with("fc:a1:3e") || p.starts_with("84:74:2a") || p.starts_with("a8:06:00") ||
             p.starts_with("c8:78:2d") || p.starts_with("cc:3a:61") || p.starts_with("e4:7c:f5") => {
                 ("Samsung".to_string(), "mobile".to_string())
             }
        
        p if p.starts_with("00:03:47") || p.starts_with("00:04:23") || p.starts_with("00:08:75") ||
             p.starts_with("00:13:e8") || p.starts_with("00:15:00") || p.starts_with("00:16:ea") ||
             p.starts_with("00:18:de") || p.starts_with("00:1b:77") || p.starts_with("00:1c:c0") ||
             p.starts_with("00:1e:64") || p.starts_with("00:21:5c") || p.starts_with("00:21:6a") ||
             p.starts_with("00:22:fb") || p.starts_with("00:23:14") || p.starts_with("00:24:d6") ||
             p.starts_with("00:28:f8") || p.starts_with("1c:bf:c0") || p.starts_with("28:18:78") ||
             p.starts_with("3c:6a:9d") || p.starts_with("4c:34:88") || p.starts_with("5c:c5:d4") ||
             p.starts_with("60:57:18") || p.starts_with("70:cd:0d") || p.starts_with("94:b4:0f") ||
             p.starts_with("a0:a8:cd") || p.starts_with("a0:c5:89") || p.starts_with("c8:ff:28") ||
             p.starts_with("e4:a7:a0") || p.starts_with("e8:b1:fc") || p.starts_with("f8:28:19") ||
             p.starts_with("f8:e9:4e") || p.starts_with("f8:75:a4") || p.starts_with("98:54:1b") => {
                 ("Intel (PC/Laptop)".to_string(), "laptop".to_string())
             }

        p if p.starts_with("00:e0:4c") || p.starts_with("00:18:1a") || p.starts_with("00:22:6b") ||
             p.starts_with("b8:27:eb") || p.starts_with("30:5a:3a") || p.starts_with("b0:4e:26") ||
             p.starts_with("74:da:38") || p.starts_with("d8:fe:e3") || p.starts_with("44:8a:5b") => {
                 ("Realtek".to_string(), "pc".to_string())
             }

        p if p.starts_with("00:14:78") || p.starts_with("00:1d:0f") || p.starts_with("00:21:29") ||
             p.starts_with("50:c7:bf") || p.starts_with("74:ea:3a") || p.starts_with("84:16:f9") ||
             p.starts_with("98:de:d0") || p.starts_with("a4:2b:b0") || p.starts_with("c5:3f:b5") ||
             p.starts_with("cc:32:e5") || p.starts_with("ec:17:2f") || p.starts_with("f8:35:dd") ||
             p.starts_with("f8:d1:11") || p.starts_with("00:1f:33") || p.starts_with("00:26:88") ||
             p.starts_with("00:1e:e5") || p.starts_with("00:24:b2") || p.starts_with("18:a6:f7") ||
             p.starts_with("30:b5:c2") || p.starts_with("c0:c9:e3") || p.starts_with("c0:4a:00") ||
             p.starts_with("e8:94:f6") || p.starts_with("f4:ec:38") => {
                 ("TP-Link / Network Device".to_string(), "router".to_string())
             }

        p if p.starts_with("00:9e:c8") || p.starts_with("1c:15:1f") || p.starts_with("28:6c:07") ||
             p.starts_with("34:80:b3") || p.starts_with("3c:bd:3e") || p.starts_with("5c:63:bf") ||
             p.starts_with("64:09:80") || p.starts_with("6c:dd:bc") || p.starts_with("7c:1d:d9") ||
             p.starts_with("9c:99:a0") || p.starts_with("a4:50:46") || p.starts_with("ac:c1:ee") ||
             p.starts_with("c4:0b:cb") || p.starts_with("d4:61:9d") || p.starts_with("e4:46:da") ||
             p.starts_with("f8:a4:5f") || p.starts_with("fc:64:ba") || p.starts_with("74:51:ba") ||
             p.starts_with("cc:2d:83") || p.starts_with("e0:19:1d") || p.starts_with("ec:d0:9f") => {
                 ("Xiaomi".to_string(), "mobile".to_string())
             }
             
        p if p.starts_with("00:18:82") || p.starts_with("00:2e:c7") || p.starts_with("00:e0:fc") ||
             p.starts_with("10:1b:54") || p.starts_with("24:df:6a") || p.starts_with("28:31:52") ||
             p.starts_with("34:2e:b4") || p.starts_with("3c:f8:08") || p.starts_with("4c:1f:cc") ||
             p.starts_with("4c:f9:5d") || p.starts_with("54:89:98") || p.starts_with("5c:4c:a9") ||
             p.starts_with("5c:b3:95") || p.starts_with("64:16:f0") || p.starts_with("78:d7:52") ||
             p.starts_with("80:b6:86") || p.starts_with("84:a8:e4") || p.starts_with("88:53:d4") ||
             p.starts_with("8c:18:d9") || p.starts_with("9c:c1:72") || p.starts_with("a4:16:32") ||
             p.starts_with("a8:ca:7b") || p.starts_with("ac:e2:15") || p.starts_with("b4:15:13") ||
             p.starts_with("bc:76:70") || p.starts_with("c8:8d:83") || p.starts_with("d4:40:f0") ||
             p.starts_with("e0:24:7f") || p.starts_with("e4:ca:12") || p.starts_with("e8:08:8b") ||
             p.starts_with("f8:e8:11") || p.starts_with("fc:48:ef") || p.starts_with("20:08:ed") ||
             p.starts_with("f0:1f:af") => {
                 ("Huawei".to_string(), "mobile".to_string())
             }
             
        p if p.starts_with("00:04:0e") || p.starts_with("00:0f:20") || p.starts_with("00:11:0a") ||
             p.starts_with("00:17:a4") || p.starts_with("00:18:71") || p.starts_with("00:1a:4b") ||
             p.starts_with("00:1e:0b") || p.starts_with("00:21:5a") || p.starts_with("00:22:64") ||
             p.starts_with("00:23:47") || p.starts_with("00:24:81") || p.starts_with("00:25:61") ||
             p.starts_with("00:26:55") || p.starts_with("00:30:c1") || p.starts_with("08:11:96") ||
             p.starts_with("10:60:4b") || p.starts_with("18:a9:05") || p.starts_with("24:be:05") ||
             p.starts_with("3c:35:56") || p.starts_with("40:b0:34") || p.starts_with("4c:d9:8f") ||
             p.starts_with("50:65:f3") || p.starts_with("5c:f3:70") || p.starts_with("70:5a:0f") ||
             p.starts_with("74:e6:e2") || p.starts_with("8c:ec:4b") || p.starts_with("98:4b:4a") ||
             p.starts_with("a4:5d:36") || p.starts_with("b0:5a:da") || p.starts_with("b4:b5:2f") ||
             p.starts_with("c8:c7:0f") || p.starts_with("cc:10:e1") || p.starts_with("d4:c9:3c") ||
             p.starts_with("d8:97:90") || p.starts_with("e4:11:5b") || p.starts_with("e8:39:35") ||
             p.starts_with("fc:15:b4") || p.starts_with("f8:b1:56") || p.starts_with("fc:3f:db") => {
                 ("HP (Hewlett-Packard)".to_string(), "printer".to_string())
             }
             
        p if p.starts_with("00:00:85") || p.starts_with("00:1e:8f") || p.starts_with("00:00:48") ||
             p.starts_with("00:26:ab") || p.starts_with("00:1b:a9") || p.starts_with("00:80:77") ||
             p.starts_with("10:52:1c") || p.starts_with("10:bf:48") || p.starts_with("18:03:73") ||
             p.starts_with("20:3a:07") || p.starts_with("30:85:a9") || p.starts_with("38:1a:52") ||
             p.starts_with("48:4d:7e") || p.starts_with("54:ee:75") || p.starts_with("60:35:c0") ||
             p.starts_with("70:20:84") || p.starts_with("84:ba:3b") || p.starts_with("8c:3a:e3") ||
             p.starts_with("90:2b:34") || p.starts_with("9c:d2:1e") || p.starts_with("ac:18:26") ||
             p.starts_with("ac:d1:b8") || p.starts_with("b4:18:a8") || p.starts_with("bc:83:85") ||
             p.starts_with("c4:3a:be") || p.starts_with("cc:6d:a0") || p.starts_with("d4:c1:fc") ||
             p.starts_with("e0:28:6d") || p.starts_with("e0:9d:31") || p.starts_with("e0:cb:ee") ||
             p.starts_with("e8:9d:87") || p.starts_with("fc:b1:cd") || p.starts_with("fc:f8:ae") ||
             p.starts_with("14:2d:f5") || p.starts_with("30:05:5c") || p.starts_with("4c:11:bf") ||
             p.starts_with("54:13:79") || p.starts_with("80:56:f2") || p.starts_with("90:9a:4a") ||
             p.starts_with("a8:66:7f") || p.starts_with("b4:75:0e") || p.starts_with("c8:60:00") ||
             p.starts_with("d0:50:99") || p.starts_with("e0:22:04") || p.starts_with("e0:c4:7a") ||
             p.starts_with("e8:07:bf") || p.starts_with("fc:c2:de") || p.starts_with("00:1a:11") ||
             p.starts_with("00:03:c5") || p.starts_with("00:22:58") || p.starts_with("3c:2a:f4") ||
             p.starts_with("40:b0:fa") || p.starts_with("48:2c:6a") || p.starts_with("54:e6:fc") => {
                 ("Printer (Epson/Canon/Brother)".to_string(), "printer".to_string())
             }

        _ => ("Dispositivo Genérico".to_string(), "unknown".to_string())
    }
}

fn guess_device_type_from_hostname(hostname: &str, default_type: &str) -> String {
    let h = hostname.to_lowercase();

    // ===== SMARTPHONES - Xiaomi / Redmi / Poco / MIUI =====
    let is_xiaomi_phone = h.contains("redmi") || h.contains("poco") || h.contains("mi-") ||
        h.contains("xiaomi") || h.contains("miui") || h.starts_with("mi") && (h.contains("-note") || h.contains("-mix") || h.contains("-pad"));

    // ===== SMARTPHONES - Samsung =====
    let samsung_patterns = [
        "galaxy", "samsung", "samsung-s", "samsung-a", "samsung-m", "samsung-f",
        "sm-a", "sm-g", "sm-m", "sm-f", "sm-n",       // Samsung model codes
        "-a54", "-a53", "-a52", "-a51", "-a50",
        "-a34", "-a33", "-a32", "-a31", "-a30",
        "-a24", "-a23", "-a22", "-a21", "-a20",
        "-a14", "-a13", "-a12", "-a11", "-a10",
        "-s21", "-s22", "-s23", "-s24", "-s25",
        "-s10", "-s20", "-note10", "-note20",
        "a54-de-", "a53-de-", "a52-de-", "a34-de-", "a33-de-",  // common Spanish hostnames
    ];
    let is_samsung_phone = samsung_patterns.iter().any(|p| h.contains(p));

    // ===== SMARTPHONES - Motorola =====
    let moto_patterns = ["moto", "motorola", "-g54", "-g53", "-g52", "-g51", "-g50",
        "-g32", "-g31", "-g30", "-g22", "-g14", "-g13", "-g10",
        "-e40", "-e32", "-edge", "moto-e", "moto-g"];
    let is_moto_phone = moto_patterns.iter().any(|p| h.contains(p));

    // ===== SMARTPHONES - OnePlus / Oppo / Realme / Vivo =====
    let other_phone_patterns = [
        "oneplus", "oppo", "realme", "vivo", "iqoo",
        "iphone", "android", "phone", "celular", "movil", "smartphone",
        "huawei-p", "huawei-y", "honor-", "nova-",
        "nokia-", "infinix", "tecno-", "itel-",
        "pixel-", "pixel",
    ];
    let is_other_phone = other_phone_patterns.iter().any(|p| h.contains(p));

    // ===== SMARTPHONES - generic patterns (5g/4g suffix, -pro, model-number-like) =====
    let has_5g = h.ends_with("-5g") || h.ends_with("-4g") || h.contains("-5g-") || h.contains("-4g-");
    let has_pro_phone = (h.ends_with("-pro") || h.ends_with("-pro-5g") || h.ends_with("-ultra")) &&
        (is_xiaomi_phone || is_samsung_phone || is_moto_phone || is_other_phone);

    if is_xiaomi_phone || is_samsung_phone || is_moto_phone || is_other_phone || has_5g || has_pro_phone {
        return "mobile".to_string();
    }

    // ===== LAPTOPS =====
    let laptop_patterns = [
        "laptop", "notebook", "macbook", "thinkpad", "zenbook", "vivobook",
        "inspiron", "latitude", "xps", "pavilion", "elitebook", "probook",
        "spectre", "envy", "yoga", "ideapad", "legion", "rog", "zephyrus",
        "tuf-", "swift-", "aspire", "nitro-", "predator", "surface",
        "chromebook", "matebook",
    ];
    if laptop_patterns.iter().any(|p| h.contains(p)) {
        return "laptop".to_string();
    }

    // ===== SMART TVs =====
    let tv_patterns = [
        "smart-tv", "smarttv", "television", "bravia", "qled",
        "roku", "chromecast", "firestick", "fire-tv", "appletv",
        "androidtv", "android-tv", "googletv", "webos",
        "lg-tv", "samsung-tv", "sony-tv", "tcl-tv", "hisense",
    ];
    // "tv" alone can be ambiguous, only match if paired with known patterns
    if tv_patterns.iter().any(|p| h.contains(p)) ||
        (h.contains("tv") && (h.contains("lg") || h.contains("sony") || h.contains("tcl") || h.contains("hisense") || h.contains("samsung"))) {
        return "tv".to_string();
    }

    // ===== PRINTERS =====
    let printer_patterns = [
        "printer", "impresora", "epson", "canon", "brother",
        "laserjet", "deskjet", "officejet", "pixma", "ecotank",
        "hl-", "mfc-", "hp-",
    ];
    if printer_patterns.iter().any(|p| h.contains(p)) {
        return "printer".to_string();
    }

    // ===== ROUTERS / NETWORK DEVICES =====
    let router_patterns = [
        "router", "gateway", "modem", "switch", "accesspoint", "ap-",
        "tplink", "tp-link", "archer", "deco",
        "zte", ".zte.", "zte.com",
        "huawei-hg", "huawei-b", "huawei-e",
        "tenda", "d-link", "dlink", "netgear", "linksys", "asus-rt",
        "mikrotik", "ubiquiti", "unifi", "edgerouter",
        "fritz", "fritzbox", "vodafone-station", "movistar-",
        "csp1.",   // ZTE-like CSP hostnames
        ".cn",    // Chinese domain suffix often routers/IoT
        "lan.", ".local.lan",
        "iskratel", "sagemcom", "technicolor", "arris",
    ];
    if router_patterns.iter().any(|p| h.contains(p)) {
        return "router".to_string();
    }

    // ===== PCs / DESKTOPS =====
    let pc_patterns = [
        "desktop", "workstation", "computadora", "torre",
        "pc-", "-pc", "desktop-",
    ];
    // "pc" alone is too common in names; only match with separator
    if pc_patterns.iter().any(|p| h.contains(p)) {
        return "pc".to_string();
    }

    default_type.to_string()
}

/// Infer the brand of a device from its hostname when the MAC OUI lookup returns the generic fallback.
fn infer_brand_from_hostname(hostname: &str) -> String {
    let h = hostname.to_lowercase();

    // Xiaomi ecosystem
    if h.contains("redmi") || h.contains("poco") || h.contains("xiaomi") || h.contains("miui") {
        return "Xiaomi".to_string();
    }
    // Samsung
    if h.contains("samsung") || h.contains("galaxy") || h.contains("sm-a") || h.contains("sm-g") ||
       h.contains("sm-m") || h.contains("sm-f") || h.contains("sm-n") ||
       h.contains("-a54") || h.contains("-a52") || h.contains("-a53") || h.contains("-a34") ||
       h.contains("-a33") || h.contains("-a32") || h.contains("-a24") || h.contains("-a23") ||
       h.contains("-a14") || h.contains("-a13") || h.contains("-s21") || h.contains("-s22") ||
       h.contains("-s23") || h.contains("-s24") || h.starts_with("a54-") || h.starts_with("a52-") {
        return "Samsung".to_string();
    }
    // Motorola
    if h.contains("moto") || h.contains("motorola") {
        return "Motorola".to_string();
    }
    // Apple
    if h.contains("iphone") || h.contains("ipad") || h.contains("macbook") || h.contains("apple") {
        return "Apple".to_string();
    }
    // Huawei / Honor
    if h.contains("huawei") || h.contains("honor-") || h.contains("nova-") {
        return "Huawei".to_string();
    }
    // OnePlus
    if h.contains("oneplus") {
        return "OnePlus".to_string();
    }
    // Oppo / Realme
    if h.contains("oppo") { return "Oppo".to_string(); }
    if h.contains("realme") { return "Realme".to_string(); }
    if h.contains("vivo") || h.contains("iqoo") { return "Vivo".to_string(); }
    if h.contains("nokia") { return "Nokia".to_string(); }
    if h.contains("pixel") { return "Google".to_string(); }
    // Network devices
    if h.contains("zte") || h.contains("csp1.") || h.ends_with(".cn") {
        return "ZTE".to_string();
    }
    if h.contains("tplink") || h.contains("tp-link") || h.contains("archer") || h.contains("deco") {
        return "TP-Link".to_string();
    }
    if h.contains("tenda") { return "Tenda".to_string(); }
    if h.contains("dlink") || h.contains("d-link") { return "D-Link".to_string(); }
    if h.contains("netgear") { return "Netgear".to_string(); }
    if h.contains("linksys") { return "Linksys".to_string(); }
    if h.contains("asus-rt") || h.contains("asus") { return "ASUS".to_string(); }
    if h.contains("mikrotik") { return "MikroTik".to_string(); }
    if h.contains("ubiquiti") || h.contains("unifi") { return "Ubiquiti".to_string(); }
    if h.contains("fritz") { return "AVM Fritz!Box".to_string(); }
    // Printers
    if h.contains("epson") { return "Epson".to_string(); }
    if h.contains("canon") { return "Canon".to_string(); }
    if h.contains("brother") { return "Brother".to_string(); }
    if h.contains("hp-") || h.contains("laserjet") || h.contains("deskjet") || h.contains("officejet") {
        return "HP".to_string();
    }
    // Desktops / PCs
    if h.contains("dell") { return "Dell".to_string(); }
    if h.contains("lenovo") || h.contains("thinkpad") || h.contains("ideapad") || h.contains("legion") {
        return "Lenovo".to_string();
    }
    if h.contains("hp") || h.contains("pavilion") || h.contains("elitebook") || h.contains("spectre") {
        return "HP".to_string();
    }
    if h.contains("acer") || h.contains("aspire") || h.contains("nitro") || h.contains("predator") {
        return "Acer".to_string();
    }
    if h.contains("rog") || h.contains("zenbook") || h.contains("vivobook") || h.contains("zephyrus") {
        return "ASUS".to_string();
    }
    if h.contains("msi") { return "MSI".to_string(); }

    "Desconocido".to_string()
}

#[tauri::command]
async fn start_free_discovery() -> Result<Vec<DiscoveredDevice>, String> {
    let local_ips = get_all_local_ips();
    info!("Iniciando escaneo de red (ping sweep) para subredes locales: {:?}", local_ips);

    let mut sweep_tasks = Vec::new();
    for ip in local_ips {
        sweep_tasks.push(tokio::spawn(async move {
            sweep_subnet(ip).await;
        }));
    }
    for task in sweep_tasks {
        let _ = task.await;
    }

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
        } else {
            if let Ok(output) = std::process::Command::new("arp").arg("-an").output() {
                arp_output = String::from_utf8_lossy(&output.stdout).to_string();
            } else if let Ok(output) = std::process::Command::new("arp").arg("-a").output() {
                arp_output = String::from_utf8_lossy(&output.stdout).to_string();
            }
        }
    }

    let arp_devices = parse_devices_from_arp_output(&arp_output);
    info!("Población de tabla ARP completada. Parseados {} dispositivos. Iniciando resolución paralela...", arp_devices.len());

    let mut device_details = Vec::new();
    let mut resolve_tasks = Vec::new();
    
    for (ip, mac) in arp_devices {
        resolve_tasks.push(tokio::spawn(async move {
            let hostname = resolve_hostname(&ip).await.unwrap_or_else(|| "unknown".to_string());
            let (mac_brand, default_type) = get_brand_from_mac(&mac);
            let device_type = guess_device_type_from_hostname(&hostname, &default_type);

            // Infer brand from hostname when MAC lookup returns the generic fallback
            let brand = if mac_brand == "Dispositivo Genérico" {
                infer_brand_from_hostname(&hostname)
            } else {
                mac_brand
            };

            let last_octet = ip.split('.').last().unwrap_or("");
            let name = if hostname == "unknown" {
                format!("Dispositivo ({})", last_octet)
            } else {
                // Clean up hostname: remove domain suffixes for display
                let display = hostname
                    .trim_end_matches(".local")
                    .trim_end_matches(".home")
                    .trim_end_matches(".lan")
                    .to_string();
                display
            };

            // Build a friendly description
            let description = match (hostname == "unknown", brand == "Desconocido") {
                (true, _)  => format!("MAC: {} • Sin nombre resuelto", mac),
                (false, true)  => format!("IP: {}", ip),
                (false, false) => format!("{} • IP: {}", brand, ip),
            };

            DiscoveredDevice {
                ip,
                mac,
                hostname: name,
                device_type,
                brand,
                description,
            }
        }));
    }
    
    for task in resolve_tasks {
        if let Ok(device) = task.await {
            device_details.push(device);
        }
    }

    device_details.sort_by(|a, b| a.ip.cmp(&b.ip));
    info!("Búsqueda libre completada. Dispositivos resueltos: {:?}", device_details);
    Ok(device_details)
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

/// Returns the local IPv4 addresses of this machine along with the hostname
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

/// Pings a given host (IP or domain) and returns latency in milliseconds
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

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        cmd.output()
    ).await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
            // Parse RTT: Windows says "time=10ms" or "tiempo=10ms", Linux says "time=10.1 ms"
            let latency = parse_ping_latency(&stdout);
            Ok(PingResult {
                host: target,
                latency_ms: latency,
                success: latency.is_some(),
                error: if latency.is_none() { Some("Sin respuesta".to_string()) } else { None },
            })
        }
        Ok(Err(e)) => Ok(PingResult {
            host: target,
            latency_ms: None,
            success: false,
            error: Some(e.to_string()),
        }),
        Err(_) => Ok(PingResult {
            host: target,
            latency_ms: None,
            success: false,
            error: Some("Timeout".to_string()),
        }),
    }
}

fn parse_ping_latency(output: &str) -> Option<f64> {
    // Match patterns like: time=12ms  tiempo=12ms  time=12.5 ms  temps=12ms
    for line in output.lines() {
        // Windows: "tiempo=12ms" or "time=12ms"
        if let Some(pos) = line.find("tiempo=").or_else(|| line.find("time=")) {
            let rest = &line[pos..];
            // Skip past "tiempo=" or "time="
            let after = rest.splitn(2, '=').nth(1)?;
            // Grab the numeric part
            let num_str: String = after.chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(ms) = num_str.parse::<f64>() {
                return Some(ms);
            }
        }
        // Linux: "time=12.5 ms"
        if line.contains("time=") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in &parts {
                if part.starts_with("time=") {
                    let num_str: String = part[5..].chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    if let Ok(ms) = num_str.parse::<f64>() {
                        return Some(ms);
                    }
                }
            }
        }
    }
    None
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
            get_local_ips,
            ping_host,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
