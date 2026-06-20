use log::info;

use crate::config::PeerInfo;
use mdns_sd::{ServiceDaemon, ServiceInfo, ServiceEvent};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Servicio de descubrimiento de peers en la red local usando mDNS
/// Servicio de descubrimiento de peers en la red local usando mDNS y UDP Broadcast
pub struct DiscoveryService {
    mdns: Option<ServiceDaemon>,
    udp_socket: Option<std::net::UdpSocket>,
    peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    #[allow(dead_code)]
    service_name: &'static str,
    service_type: &'static str,
    running: Arc<Mutex<bool>>,
}

impl DiscoveryService {
    /// Crear nuevo servicio de descubrimiento
    pub fn new() -> Self {
        Self {
            mdns: None,
            udp_socket: None,
            peers: Arc::new(Mutex::new(HashMap::new())),
            service_name: "_pcconector._tcp.local.",
            service_type: "_pcconector._tcp.local.",
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Iniciar el descubrimiento: registra este dispositivo y comienza a buscar otros
    pub fn start(&mut self, hostname: &str, port: u16) -> Result<(), String> {
        let mut running_guard = self.running.lock().unwrap();
        if *running_guard {
            return Ok(());
        }
        *running_guard = true;

        // Detect local IPs to avoid discovering ourselves
        let mut local_ips = vec!["127.0.0.1".to_string(), "0.0.0.0".to_string()];
        if let Ok(interfaces) = get_if_addrs::get_if_addrs() {
            for iface in interfaces {
                if let std::net::IpAddr::V4(ip) = iface.ip() {
                    local_ips.push(ip.to_string());
                }
            }
        }
        let local_ips = Arc::new(local_ips);

        // 1. INICIAR mDNS DISCOVERY
        if let Ok(mdns) = ServiceDaemon::new() {
            let service_info = ServiceInfo::new(
                self.service_type,
                hostname,
                &format!("{}.local.", hostname),
                "", // IP se detecta automáticamente
                port,
                None, // sin propiedades adicionales
            );
            if let Ok(info) = service_info {
                let info = info.enable_addr_auto();
                if mdns.register(info).is_ok() {
                    info!("Servicio mDNS registrado: {} en puerto {}", hostname, port);
                    if let Ok(receiver) = mdns.browse(self.service_type) {
                        let peers_clone = self.peers.clone();
                        let local_ips_mdns = local_ips.clone();
                        std::thread::spawn(move || {
                            for event in receiver {
                                match event {
                                    ServiceEvent::ServiceResolved(info) => {
                                        let name = info.get_hostname().trim_end_matches(".local.").to_string();
                                        let ip_address = info.get_addresses()
                                            .iter()
                                            .find(|addr| addr.is_ipv4())
                                            .map(|a| a.to_string())
                                            .unwrap_or_else(|| {
                                                info.get_addresses()
                                                    .iter()
                                                    .next()
                                                    .map(|a| a.to_string())
                                                    .unwrap_or_default()
                                            });

                                        if local_ips_mdns.contains(&ip_address) {
                                            continue;
                                        }

                                        let peer = PeerInfo {
                                            id: info.get_fullname().to_string(),
                                            name: name.clone(),
                                            hostname: name,
                                            ip_address,
                                            port: info.get_port(),
                                            os: String::new(),
                                            version: String::new(),
                                        };
                                        info!("Peer mDNS descubierto: {} en {}", peer.name, peer.ip_address);
                                        let mut peers = peers_clone.lock().unwrap();
                                        peers.insert(peer.id.clone(), peer);
                                    }
                                    ServiceEvent::ServiceRemoved(_service_type, fullname) => {
                                        info!("Peer mDNS eliminado: {}", fullname);
                                        let mut peers = peers_clone.lock().unwrap();
                                        peers.remove(&fullname);
                                    }
                                    _ => {}
                                }
                            }
                        });
                        self.mdns = Some(mdns);
                    }
                }
            }
        }

        // 2. INICIAR UDP BROADCAST (Puerto 9875)
        let socket = std::net::UdpSocket::bind("0.0.0.0:9875")
            .map_err(|e| format!("Error al enlazar socket UDP Broadcast 9875: {}", e))?;
        socket.set_broadcast(true).map_err(|e| e.to_string())?;
        socket.set_nonblocking(false).map_err(|e| e.to_string())?;

        let socket_clone = socket.try_clone().map_err(|e| e.to_string())?;
        let peers_udp = self.peers.clone();
        let running_clone = self.running.clone();
        let hostname_str = hostname.to_string();
        let local_ips_udp = local_ips.clone();

        // Spawnea receptor UDP
        std::thread::spawn(move || {
            let mut buf = [0u8; 1024];
            info!("Receptor UDP Broadcast escuchando en puerto 9875...");
            while *running_clone.lock().unwrap() {
                match socket_clone.recv_from(&mut buf) {
                    Ok((len, src)) => {
                        let msg = String::from_utf8_lossy(&buf[..len]).trim().to_string();
                        let src_ip = src.ip().to_string();

                        // Ignorar paquetes de nosotros mismos
                        if local_ips_udp.contains(&src_ip) {
                            continue;
                        }

                        if msg.starts_with("NETBRIDGE_PING:") {
                            let remote_host = msg.trim_start_matches("NETBRIDGE_PING:").to_string();
                            
                            // Registrar peer descubierto
                            let peer_id = format!("udp-{}", src_ip);
                            {
                                let mut peers = peers_udp.lock().unwrap();
                                peers.insert(peer_id.clone(), PeerInfo {
                                    id: peer_id,
                                    name: remote_host.clone(),
                                    hostname: remote_host.clone(),
                                    ip_address: src_ip.clone(),
                                    port: 9876,
                                    os: String::new(),
                                    version: String::new(),
                                });
                            }

                            // Responder PONG
                            let pong_msg = format!("NETBRIDGE_PONG:{}", hostname_str);
                            let _ = socket_clone.send_to(pong_msg.as_bytes(), src);
                        } else if msg.starts_with("NETBRIDGE_PONG:") {
                            let remote_host = msg.trim_start_matches("NETBRIDGE_PONG:").to_string();
                            let peer_id = format!("udp-{}", src_ip);
                            let mut peers = peers_udp.lock().unwrap();
                            peers.insert(peer_id.clone(), PeerInfo {
                                id: peer_id,
                                name: remote_host,
                                hostname: hostname_str.clone(),
                                ip_address: src_ip,
                                port: 9876,
                                os: String::new(),
                                version: String::new(),
                            });
                        }
                    }
                    Err(_) => {
                        // El socket se cerró al hacer drop o shutdown
                        break;
                    }
                }
            }
            info!("Receptor UDP Broadcast finalizado.");
        });

        // Spawnea transmisor UDP (Broadcast ping cada 3 segundos)
        let socket_send = socket.try_clone().map_err(|e| e.to_string())?;
        let running_send = self.running.clone();
        let ping_msg = format!("NETBRIDGE_PING:{}", hostname);
        std::thread::spawn(move || {
            info!("Transmisor UDP Broadcast iniciado...");
            while *running_send.lock().unwrap() {
                let _ = socket_send.send_to(ping_msg.as_bytes(), "255.255.255.255:9875");
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
            info!("Transmisor UDP Broadcast finalizado.");
        });

        self.udp_socket = Some(socket);
        Ok(())
    }

    /// Obtener lista de peers descubiertos
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.lock().unwrap().values().cloned().collect()
    }

    /// Detener el servicio de descubrimiento
    pub fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
        if let Some(mdns) = self.mdns.take() {
            mdns.shutdown().ok();
            info!("Servicio mDNS detenido");
        }
        if let Some(socket) = self.udp_socket.take() {
            // Provocar que recv_from salga inmediatamente cerrando el socket
            // En algunos sistemas, simplemente dejarlo caer (drop) no desbloquea recv_from inmediatamente,
            // pero enviar un ping local o hacer un bind ficticio ayuda. dropsock es el estándar.
            // Para asegurar cierre en Windows/Linux, intentamos hacer un send ficticio a nosotros mismos.
            let _ = socket.send_to(b"SHUTDOWN", "127.0.0.1:9875");
            info!("Servicio UDP Broadcast detenido");
        }
        self.peers.lock().unwrap().clear();
    }
}

impl Drop for DiscoveryService {
    fn drop(&mut self) {
        self.stop();
    }
}
