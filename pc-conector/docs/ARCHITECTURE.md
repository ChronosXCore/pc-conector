# PC Conector - Architecture

## 🏗️ Arquitectura General

```
┌─────────────────────────────────────────────────────────────┐
│                    PC Conector App                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Frontend (React + TypeScript)               │  │
│  │  - UI de configuración                                │  │
│  │  - Panel de dispositivos                              │  │
│  │  - Visualización de conexión                         │  │
│  └──────────────────────┬───────────────────────────────┘  │
│                          │ Tauri Commands (IPC)             │
│  ┌──────────────────────┴───────────────────────────────┐  │
│  │           Backend (Rust)                              │  │
│  │                                                       │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌────────────┐  │  │
│  │  │ Discovery   │  │ Network      │  │ Clipboard  │  │  │
│  │  │ (mDNS)      │  │ (TCP/QUIC)   │  │ Sync       │  │  │
│  │  └─────────────┘  └──────────────┘  └────────────┘  │  │
│  │                                                       │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌────────────┐  │  │
│  │  │ Input       │  │ Audio        │  │ Config     │  │  │
│  │  │ Capture     │  │ Streaming    │  │ Manager    │  │  │
│  │  │ (rdev)      │  │ (cpal)       │  │            │  │  │
│  │  └─────────────┘  └──────────────┘  └────────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
│                            │                                │
│                    Network (WiFi LAN)                       │
└─────────────────────────────────────────────────────────────┘
```

## 📡 Componentes Principales

### 1. Discovery Module
- Utiliza **mDNS** (Multicast DNS) para descubrimiento automático de peers
- Escucha en la red local para encontrar otras instancias de PC Conector
- Intercambia información básica (nombre del host, IP, capacidades)

### 2. Network Module
- Comunicación **P2P** sobre TCP con cifrado
- Baja latencia optimizada para LAN
- Reconexión automática ante caídas de red

### 3. Clipboard Sync
- Monitorea cambios en el portapapeles local
- Envía actualizaciones al peer conectado
- Recibe y aplica actualizaciones del peer

### 4. Input Capture (Mouse/Teclado)
- Captura eventos globales de teclado y mouse usando `rdev`
- Envía eventos al peer como si fueran entradas locales
- Maneja el movimiento de mouse entre pantallas configuradas

### 5. Audio Streaming
- Captura audio del micrófono usando `cpal`
- Reproduce audio recibido en los altavoces
- Streaming bidireccional con compresión para baja latencia

### 6. Config Manager
- Almacena configuración en JSON local
- Gestión de perfiles de conexión
- Configuración de auto-inicio
- Configuración de dispositivos de audio
