use log::info;

use crate::config::PeerInfo;
use mdns_sd::{ServiceDaemon, ServiceInfo, ServiceEvent};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Servicio de descubrimiento de peers en la red local usando mDNS
pub struct DiscoveryService {
    mdns: Option<ServiceDaemon>,
    peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    #[allow(dead_code)]
    service_name: &'static str,
    service_type: &'static str,
}

impl DiscoveryService {
    /// Crear nuevo servicio de descubrimiento
    pub fn new() -> Self {
        Self {
            mdns: None,
            peers: Arc::new(Mutex::new(HashMap::new())),
            service_name: "_pcconector._tcp.local.",
            service_type: "_pcconector._tcp.local.",
        }
    }

    /// Iniciar el descubrimiento: registra este dispositivo y comienza a buscar otros
    pub fn start(&mut self, hostname: &str, port: u16) -> Result<(), String> {
        let mdns = ServiceDaemon::new().map_err(|e| format!("Error al iniciar mDNS: {}", e))?;

        // Registrar este dispositivo como servicio disponible
        let service_info = ServiceInfo::new(
            self.service_type,
            hostname,
            &format!("{}.local.", hostname),
            "", // IP se detecta automáticamente
            port,
            None, // sin propiedades adicionales
        ).map_err(|e| format!("Error al crear servicio mDNS: {}", e))?;

        mdns.register(service_info)
            .map_err(|e| format!("Error al registrar servicio: {}", e))?;

        info!("Servicio mDNS registrado: {} en puerto {}", hostname, port);

        // Empezar a buscar otros peers
        let receiver = mdns.browse(self.service_type)
            .map_err(|e| format!("Error al iniciar búsqueda: {}", e))?;

        let peers = self.peers.clone();
        std::thread::spawn(move || {
            for event in receiver {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let peer = PeerInfo {
                            id: info.get_fullname().to_string(),
                            name: info.get_hostname().to_string(),
                            hostname: info.get_hostname().to_string(),
                            ip_address: info.get_addresses()
                                .iter()
                                .find(|addr| addr.is_ipv4())
                                .map(|a| a.to_string())
                                .unwrap_or_else(|| {
                                    info.get_addresses()
                                        .iter()
                                        .next()
                                        .map(|a| a.to_string())
                                        .unwrap_or_default()
                                }),
                            port: info.get_port(),
                            os: String::new(),
                            version: String::new(),
                        };
                        info!("Peer descubierto: {} en {}", peer.name, peer.ip_address);
                        
                        let mut peers = peers.lock().unwrap();
                        peers.insert(peer.id.clone(), peer);
                    }
                    ServiceEvent::ServiceRemoved(_service_type, fullname) => {
                        info!("Peer eliminado: {}", fullname);
                        let mut peers = peers.lock().unwrap();
                        peers.remove(&fullname);
                    }
                    _ => {}
                }
            }
        });

        self.mdns = Some(mdns);
        Ok(())
    }

    /// Obtener lista de peers descubiertos
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.lock().unwrap().values().cloned().collect()
    }

    /// Detener el servicio de descubrimiento
    pub fn stop(&mut self) {
        if let Some(mdns) = self.mdns.take() {
            mdns.shutdown().ok();
            info!("Servicio mDNS detenido");
        }
    }
}

impl Drop for DiscoveryService {
    fn drop(&mut self) {
        self.stop();
    }
}
