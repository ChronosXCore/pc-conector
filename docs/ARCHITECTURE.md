# 🏗️ PC Conector - Arquitectura del Sistema

## Visión General de la Arquitectura

PC Conector sigue una arquitectura **cliente-servidor descentralizada**:

- Cada instancia puede actuar como **servidor** (comparte sus recursos) y **cliente** (consume recursos remotos) simultáneamente
- La comunicación es **peer-to-peer** sobre la red local
- No requiere infraestructura externa (servidores cloud, cuentas, etc.)

---

## Diagrama de Arquitectura

```
┌──────────────────────┐         ┌──────────────────────┐
│     PC A (Host)      │         │     PC B (Cliente)   │
│                      │         │                      │
│ ┌────────────────┐   │  mDNS   │ ┌────────────────┐   │
│ │ Descubrimiento │───┼─────────┼─│ Descubrimiento │   │
│ │   (mDNS)       │   │         │ │   (mDNS)       │   │
│ └────────────────┘   │         │ └────────────────┘   │
│         │            │         │         │            │
│         ▼            │         │         ▼            │
│ ┌────────────────┐   │  TCP    │ ┌────────────────┐   │
│ │ Clipboard Sync │◄──┼─────────┼─│ Clipboard Sync │   │
│ └────────────────┘   │  (WS)   │ └────────────────┘   │
│                      │         │                      │
│ ┌────────────────┐   │  UDP    │ ┌────────────────┐   │
│ │  Input Share   │◄──┼─────────┼─│  Input Share   │   │
│ │ (Captura/Simu) │   │         │ │ (Captura/Simu) │   │
│ └────────────────┘   │         │ └────────────────┘   │
│                      │         │                      │
│ ┌────────────────┐   │  UDP    │ ┌────────────────┐   │
│ │  Audio Stream  │◄──┼─────────┼─│  Audio Stream  │   │
│ │  (Codec: Opus) │   │         │ │  (Codec: Opus) │   │
│ └────────────────┘   │         │ └────────────────┘   │
│                      │         │                      │
│ ┌────────────────┐   │  TCP    │ ┌────────────────┐   │
│ │  Config Sync   │◄──┼─────────┼─│  Config Sync   │   │
│ └────────────────┘   │  (WS)   │ └────────────────┘   │
└──────────────────────┘         └──────────────────────┘
```

## Módulos Principales

### 1. Network Manager
- **Responsabilidad**: Gestionar todas las conexiones de red
- **Descubrimiento**: mDNS para encontrar otros PCs con PC Conector
- **Conexión**: WebSocket seguro para señalización y datos
- **Detección de desconexión**: Heartbeats cada 5 segundos

### 2. Clipboard Sync
- **Responsabilidad**: Sincronizar el portapapeles entre PCs
- **Monitor**: Observa cambios en el portapapeles local usando polling + eventos
- **Transmisión**: Envía contenido nuevo a todos los peers conectados
- **Recepción**: Actualiza el portapapeles local con contenido remoto
- **Formatos**: Texto plano, imágenes, RTF

### 3. Input Share (Mouse + Teclado)
- **Responsabilidad**: Compartir mouse y teclado entre PCs
- **Coordenadas Virtuales**: Sistema de coordenadas global basado en la grilla de monitores
- **Captura**: rdev para capturar eventos globales de entrada
- **Simulación**: enigo para simular eventos en la máquina remota
- **Hot Corner**: Al llegar al borde de la pantalla, el cursor se transfiere al siguiente PC
- **Clipboard Bridge**: Al transferir el cursor, el portapapeles se sincroniza automáticamente

### 4. Audio Stream
- **Responsabilidad**: Transmitir audio entre PCs en tiempo real
- **Captura**: CPAL para capturar desde dispositivos de entrada/salida
- **Codec**: Opus para compresión de audio de baja latencia
- **Transmisión**: UDP con secuenciación de paquetes
- **Recepción**: Buffer de jitter para compensar variación de latencia
- **Dispositivos**: Selección configurable de dispositivos de audio

### 5. Config Module
- **Responsabilidad**: Persistir y sincronizar configuración
- **Almacenamiento**: JSON en el directorio de datos de la aplicación
- **Contenido**:
  - Dispositivos confiados (peer IDs + nombres)
  - Posición de monitores en la grilla
  - Funciones habilitadas/deshabilitadas
  - Preferencias de audio (dispositivos, calidad)
  - Auto-inicio con el sistema
- **Auto-conexión**: Conectar automáticamente al PC configurado al iniciar

---

## Flujo de Conexión

1. **Inicio**: La app se inicia y comienza a anunciarse via mDNS como "PC-Conector"
2. **Descubrimiento**: Escucha broadcasts mDNS de otros PCs con la app
3. **Lista**: Muestra los PCs descubiertos en la interfaz
4. **Conexión**: El usuario (o auto-conexión) inicia conexión WebSocket
5. **Handshake**: Intercambio de capacidades, configuraciones y posición de monitores
6. **Operación**: Canales de datos separados para clipboard, input y audio
7. **Desconexión**: Heartbeat perdido → reconexión automática

---

## Puertos por Defecto

| Puerto | Protocolo | Servicio |
|--------|-----------|---------|
| 5353 | UDP | mDNS (descubrimiento) |
| 24800 | TCP | WebSocket (señalización y datos) |
| 24801-24810 | UDP | Streaming de audio |
