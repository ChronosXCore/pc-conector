<div align="center">

# 🛠️ PC Conector — Stack Tecnológico Detallado

[![Back to README](https://img.shields.io/badge/←_Volver_al_README-gray?style=flat-square)](../README.md)

</div>

---

## 🎨 Frontend

### Tecnologías Principales

| Tecnología | Versión | Propósito |
|-----------|:-------:|-----------|
| ![React](https://img.shields.io/badge/React-^19-61dafb?logo=react&logoColor=black&style=flat-square) | ^19 | Framework de UI |
| ![TypeScript](https://img.shields.io/badge/TypeScript-~6.0-3178c6?logo=typescript&logoColor=white&style=flat-square) | ~6.0 | Tipado estático y DX superior |
| ![Vite](https://img.shields.io/badge/Vite-^8.0-646cff?logo=vite&logoColor=white&style=flat-square) | ^8.0 | Build tool / Dev server ultrarrápido |
| **CSS Puro** | — | Glassmorphism, animaciones, temas |

### Dependencias Frontend

| Paquete | Propósito |
|---------|-----------|
| `@tauri-apps/api` | Comunicación bidireccional con el backend Rust |
| `react-router-dom` | Navegación entre vistas de la app |
| `zustand` | Estado global reactivo (configuración de usuario) |
| `@dnd-kit/core` | Drag & drop para posicionamiento visual de monitores |

---

## 🦀 Backend (Rust / Tauri v2)

### Framework y Runtime

| Crate | Versión | Propósito |
|-------|:-------:|-----------|
| `tauri` | 2.11 | Framework de escritorio multiplataforma |
| `tokio` | latest | Runtime asíncrono de alta performance |
| `serde` / `serde_json` | latest | Serialización/deserialización de datos |
| `tracing` | latest | Logging estructurado y depuración |

### Módulos Funcionales

| Crate | Módulo | Propósito |
|-------|--------|-----------|
| `arboard` | Portapapeles | Acceso multiplataforma al portapapeles del sistema |
| `enigo` | Input Share | Simulación de eventos de mouse y teclado |
| `rdev` | Input Share | Captura de eventos globales del sistema |
| `cpal` | Audio | Captura y reproducción de audio (plataforma-nativo) |
| `symphonia` | Audio | Decodificación y streaming de audio |
| `mdns-sd` | Red | Descubrimiento automático vía mDNS/Zeroconf |
| `tokio-tungstenite` | Red | WebSockets para comunicación en tiempo real |

---

## 🌐 Protocolos de Comunicación

```mermaid
graph LR
    subgraph Protocolos["Protocolos de Red"]
        mDNS["📡 mDNS\nUDP 5353\nDescubrimiento"]
        WS["🔌 WebSocket/TLS\nTCP 24800\nPortapapeles · Control"]
        UDP["🎵 UDP Raw\n24801-24810\nAudio Opus"]
    end

    PCA["🖥️ PC A"] <-->|"Auto-descubrimiento"| mDNS
    mDNS <-->|"Auto-descubrimiento"| PCB["💻 PC B"]
    
    PCA <-->|"Datos fiables"| WS
    WS <-->|"Datos fiables"| PCB
    
    PCA <-->|"Baja latencia"| UDP
    UDP <-->|"Baja latencia"| PCB
```

| Protocolo | Puerto | Uso | Por qué |
|-----------|:------:|-----|---------|
| mDNS (RFC 6762) | 5353 | Descubrimiento automático en LAN | Sin configuración manual de IP |
| WebSocket (TLS) | 24800 | Señalización, comandos, portapapeles | Confiable, full-duplex, seguro |
| UDP | 24801-24810 | Streaming de audio | Baja latencia, no importa pérdida mínima |

---

## 🖥️ Plataformas Objetivo

| Plataforma | Soporte | Notas |
|-----------|:-------:|-------|
| ![Windows](https://img.shields.io/badge/Windows_10-0078d4?logo=windows&logoColor=white&style=flat-square) | ✅ Completo | Probado en Windows 10 y 11 |
| ![Windows](https://img.shields.io/badge/Windows_11-0078d4?logo=windows&logoColor=white&style=flat-square) | ✅ Completo | Incluye soporte para WinUI |
| ![Linux](https://img.shields.io/badge/Arch_Linux-1793d1?logo=arch-linux&logoColor=white&style=flat-square) | ✅ Completo | Omarchy Linux / CachyOS / Arch |
| ![Linux](https://img.shields.io/badge/Ubuntu-e95420?logo=ubuntu&logoColor=white&style=flat-square) | ✅ Compatible | Ubuntu 22.04+ |
| ![Linux](https://img.shields.io/badge/Fedora-51a2da?logo=fedora&logoColor=white&style=flat-square) | ✅ Compatible | Fedora 38+ |

---

## 📐 Diagrama de Capas

```
┌──────────────────────────────────────────────────────────┐
│                     PC Conector                          │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │           🎨 Frontend (React + Vite)               │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │  │
│  │  │  Config   │ │  Status  │ │   Monitor Grid   │   │  │
│  │  │   Panel   │ │  Panel   │ │  (Drag & Drop)   │   │  │
│  │  └──────────┘ └──────────┘ └──────────────────┘   │  │
│  └─────────────────────┬──────────────────────────────┘  │
│                        │ Tauri IPC                        │
│  ┌─────────────────────▼──────────────────────────────┐  │
│  │              🦀 Backend (Rust / Tauri 2)           │  │
│  │  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌────────┐  │  │
│  │  │ Network  │ │Clipboard │ │ Input  │ │ Audio  │  │  │
│  │  │ Manager  │ │  Sync    │ │ Share  │ │ Stream │  │  │
│  │  │ (mDNS+WS)│ │(arboard) │ │rdev+en │ │cpal+Op │  │  │
│  │  └──────────┘ └──────────┘ └────────┘ └────────┘  │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

---

<div align="center">

[← Volver al README](../README.md) · [Arquitectura →](ARCHITECTURE.md)

</div>
