# 🛠️ PC Conector - Stack Tecnológico Detallado

## Frontend

### React + TypeScript + Vite
| Componente | Versión | Propósito |
|-----------|---------|-----------|
| React | ^19 | UI Framework |
| TypeScript | ~6.0 | Tipado estático |
| Vite | ^8.0 | Build tool / Dev server |

### Dependencias Frontend (planificadas)
| Paquete | Propósito |
|---------|-----------|
| @tauri-apps/api | Comunicación con backend Rust |
| react-router-dom | Navegación entre vistas |
| zustand | Estado global (configuración) |
| @dnd-kit/core | Drag & drop para posiciones de monitor |

---

## Backend (Rust / Tauri v2)

### Dependencias Rust

| Crate | Propósito |
|-------|-----------|
| tauri 2.11 | Framework de escritorio |
| serde / serde_json | Serialización de datos |
| tokio | Runtime asíncrono |
| arboard | Acceso al portapapeles del sistema |
| enigo | Simulación de input (mouse/teclado) |
| cpal | Captura/reproducción de audio |
| mdns | Descubrimiento automático de dispositivos en red |
| tokio-tungstenite | WebSockets para comunicación en tiempo real |
| rdev | Captura de eventos globales de teclado/mouse |
| symphonia | Decodificación/streaming de audio |
| tracing | Logging y depuración |

---

## Comunicación en Red

| Protocolo | Uso |
|-----------|-----|
| mDNS (RFC 6762) | Descubrimiento automático de PCs en LAN |
| WebSocket (TCP) | Señalización, comandos, portapapeles |
| UDP (Raw/Protobuf) | Streaming de audio de baja latencia |
| TCP | Transferencia de datos fiables |

---

## Arquitectura

```
┌─────────────────────────────────────────────────┐
│                   PC Conector                     │
│  ┌───────────────────────────────────────────┐    │
│  │         Frontend (React + Tauri)          │    │
│  │  ┌─────────┐ ┌──────────┐ ┌──────────┐   │    │
│  │  │  Config  │ │  Status  │ │ Monitor  │   │    │
│  │  │   UI     │ │  Panel   │ │   Grid   │   │    │
│  │  └─────────┘ └──────────┘ └──────────┘   │    │
│  └───────────────────────────────────────────┘    │
│                        │                          │
│  ┌──────────────────────┴────────────────────────┐│
│  │           Backend (Rust / Tauri)              ││
│  │  ┌────────┐┌──────────┐┌─────────┐┌───────┐  ││
│  │  │Network ││ Clipboard││  Input  ││ Audio │  ││
│  │  │Manager ││  Sync    ││  Share  ││Stream │  ││
│  │  └────────┘└──────────┘└─────────┘└───────┘  ││
│  └──────────────────────────────────────────────┘│
└─────────────────────────────────────────────────┘
```

---

## Plataformas Objetivo

| Plataforma | Soporte |
|-----------|---------|
| Windows 10/11 | ✅ Completo |
| Linux (Omarchy Linux) | ✅ Completo |
| Linux (otras distros) | ✅ Compatible |
