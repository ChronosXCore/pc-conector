use crate::config::AppConfig;
use bytes::Bytes;
use log::{info, warn, error};
use quinn::{ClientConfig, Connection, Endpoint, ServerConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

/// Tipos de mensajes que se pueden enviar entre peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Clipboard(String),
    MouseEvent(MouseData),
    KeyboardEvent(KeyboardData),
    AudioData(Vec<u8>),
    AudioConfig(AudioStreamConfig),
    PeerInfo(String, String),
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseData {
    pub event_type: MouseEventType,
    pub x: f64,
    pub y: f64,
    pub button: Option<u8>,
    pub scroll_delta: Option<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseEventType {
    Move,
    Press,
    Release,
    Scroll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardData {
    pub event_type: KeyboardEventType,
    pub key_code: u32,
    pub key_char: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyboardEventType {
    Press,
    Release,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub bitrate: u32,
}

/// Gestor de comunicación QUIC entre peers
pub struct NetworkManager {
    pub endpoint: Option<Endpoint>,
    pub connection: Option<Connection>,
    config: AppConfig,
    message_tx: Option<mpsc::Sender<NetworkMessage>>,
    message_rx: Option<mpsc::Receiver<NetworkMessage>>,
}

impl NetworkManager {
    pub fn new(config: AppConfig) -> Self {
        let (tx, rx) = mpsc::channel::<NetworkMessage>(256);
        Self {
            endpoint: None,
            connection: None,
            config,
            message_tx: Some(tx),
            message_rx: Some(rx),
        }
    }

    /// Iniciar servidor QUIC (modo escucha)
    pub async fn start_server(&mut self, port: u16) -> Result<(), String> {
        let server_config = configure_server()?;
        
        let bind_addr: SocketAddr = format!("0.0.0.0:{}", port)
            .parse()
            .map_err(|e| format!("Dirección inválida: {}", e))?;

        let endpoint = Endpoint::server(server_config, bind_addr)
            .map_err(|e| format!("Error al crear endpoint: {}", e))?;

        info!("Servidor QUIC escuchando en puerto {}", port);
        self.endpoint = Some(endpoint);
        Ok(())
    }

    /// Aceptar conexiones entrantes (para modo servidor)
    pub async fn accept_connection(&mut self) -> Result<(), String> {
        if let Some(endpoint) = &self.endpoint {
            let incoming = endpoint
                .accept()
                .await
                .ok_or_else(|| "Endpoint cerrado".to_string())?;

            let conn = incoming
                .await
                .map_err(|e| format!("Error al establecer conexión entrante: {}", e))?;

            info!("Conexión entrante aceptada de {}", conn.remote_address());
            self.connection = Some(conn);
            Ok(())
        } else {
            Err("Servidor no iniciado".to_string())
        }
    }

    /// Conectar a un peer remoto (modo cliente)
    pub async fn connect(&mut self, addr: &str, port: u16) -> Result<(), String> {
        let client_config = configure_client()?;
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())
            .map_err(|e| format!("Error al crear endpoint cliente: {}", e))?;

        endpoint.set_default_client_config(client_config);

        let server_addr: SocketAddr = format!("{}:{}", addr, port)
            .parse()
            .map_err(|e| format!("Dirección del servidor inválida: {}", e))?;

        let connection = endpoint
            .connect(server_addr, "localhost")
            .map_err(|e| format!("Error al conectar: {}", e))?
            .await
            .map_err(|e| format!("Conexión rechazada: {}", e))?;

        info!("Conectado a {}:{}", addr, port);
        self.connection = Some(connection);
        self.endpoint = Some(endpoint);
        Ok(())
    }

    /// Enviar un mensaje al peer conectado
    pub async fn send_message(&self, message: &NetworkMessage) -> Result<(), String> {
        let bytes = serde_json::to_vec(message).map_err(|e| e.to_string())?;
        if let Some(conn) = &self.connection {
            let mut send = conn
                .open_uni()
                .await
                .map_err(|e| format!("Error al abrir stream uni: {}", e))?;

            send.write_all(&bytes)
                .await
                .map_err(|e| format!("Error al enviar: {}", e))?;
            send.finish();
            Ok(())
        } else {
            Err("No hay conexión activa".to_string())
        }
    }

    /// Recibir mensajes del peer (bloqueante)
    pub async fn receive_message(&self) -> Result<NetworkMessage, String> {
        if let Some(conn) = &self.connection {
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
        } else {
            Err("No hay conexión activa".to_string())
        }
    }

    /// Obtener el canal de envío de mensajes
    pub fn get_sender(&self) -> Option<mpsc::Sender<NetworkMessage>> {
        self.message_tx.clone()
    }

    /// Desconectar
    pub fn disconnect(&mut self) {
        self.connection = None;
        self.endpoint = None;
        info!("Desconectado");
    }
}

fn configure_server() -> Result<ServerConfig, String> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])
        .map_err(|e| format!("Error al generar certificado: {}", e))?;

    let cert_der = CertificateDer::from(cert.cert.der().to_vec());
    let key_der_bytes = cert.key_pair.serialize_der();
    let key_der = PrivateKeyDer::try_from(key_der_bytes)
        .map_err(|e| format!("Error en llave privada: {}", e))?;

    let server_config = ServerConfig::with_single_cert(
        vec![cert_der],
        key_der,
    ).map_err(|e| format!("Error en configuración del servidor: {}", e))?;

    Ok(server_config)
}

#[derive(Debug)]
struct SkipServerVerification;

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

fn configure_client() -> Result<ClientConfig, String> {
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
        .with_no_client_auth();

    let client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto).map_err(|e| e.to_string())?
    ));

    Ok(client_config)
}
