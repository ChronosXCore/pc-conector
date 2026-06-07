use crate::config::AppConfig;
use log::info;
use quinn::{ClientConfig, Connection, Endpoint, ServerConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

/// Screen information shared between peers after connecting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenInfo {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

/// Tipos de mensajes que se pueden enviar entre peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Clipboard(String),
    MouseEvent(MouseData),
    KeyboardEvent(KeyboardData),
    AudioData(Vec<u8>),
    AudioConfig(AudioStreamConfig),
    PeerInfo(String, String),
    ScreenLayout(Vec<ScreenInfo>),
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
    pub config: AppConfig,
}

impl NetworkManager {
    pub fn new(config: AppConfig) -> Self {
        Self {
            endpoint: None,
            config,
        }
    }

    /// Iniciar servidor QUIC (modo escucha)
    pub async fn start_server(&mut self, port: u16) -> Result<Endpoint, String> {
        let server_config = configure_server()?;
        
        let bind_addr: SocketAddr = format!("0.0.0.0:{}", port)
            .parse()
            .map_err(|e| format!("Dirección inválida: {}", e))?;

        let endpoint = Endpoint::server(server_config, bind_addr)
            .map_err(|e| format!("Error al crear endpoint: {}", e))?;

        info!("Servidor QUIC escuchando en puerto {}", port);
        self.endpoint = Some(endpoint.clone());
        Ok(endpoint)
    }

    /// Conectar a un peer remoto (modo cliente) con tiempo límite de 5 segundos
    pub async fn connect(addr: &str, port: u16) -> Result<Connection, String> {
        let client_config = configure_client()?;
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())
            .map_err(|e| format!("Error al crear endpoint cliente: {}", e))?;

        endpoint.set_default_client_config(client_config);

        let server_addr: SocketAddr = format!("{}:{}", addr, port)
            .parse()
            .map_err(|e| format!("Dirección del servidor inválida: {}", e))?;

        let connect_future = endpoint
            .connect(server_addr, "localhost")
            .map_err(|e| format!("Error al conectar: {}", e))?;

        let connection = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            connect_future,
        )
        .await
        .map_err(|_| format!("Tiempo de espera agotado: {} no respondió en 5 segundos. ¿Está PC Conector abierto en ese equipo?", addr))?
        .map_err(|e| format!("Conexión rechazada: {}", e))?;

        info!("Conectado a {}:{}", addr, port);
        Ok(connection)
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
