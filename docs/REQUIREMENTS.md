# PC Conector - Requisitos Detallados

## 1. Funcionalidades Principales

### 1.1 Descubrimiento Automático en Red
- [ ] Usar mDNS/Bonjour para detectar automáticamente otras instancias en la LAN
- [ ] Mostrar lista de PCs disponibles con nombre, IP y SO
- [ ] Conexión automática opcional basada en configuración

### 1.2 Compartir Portapapeles
- [ ] Monitorear cambios en el portapapeles local
- [ ] Enviar cambios a PCs remotos en tiempo real
- [ ] Sincronizar texto plano
- [ ] Sincronizar imágenes (opcional, configurable)
- [ ] Sincronizar archivos (opcional, configurable)
- [ ] Latencia < 200ms

### 1.3 Compartir Mouse y Teclado
- [ ] Movimiento del mouse entre pantallas (como si fuera un segundo monitor)
- [ ] Click y arrastrar entre PCs
- [ ] Teclado compartido (escribir en el PC remoto)
- [ ] Configuración de posición de pantallas (arrastrar para ordenar)
- [ ] Soporte para múltiples monitores en cada PC
- [ ] Latencia de entrada < 16ms (60fps)

### 1.4 Compartir Audio
- [ ] Transmisión de audio desde micrófono de PC A a PC B
- [ ] Transmisión de audio del sistema de PC A a PC B
- [ ] Bidireccional (ambos sentidos simultáneamente)
- [ ] Selección de dispositivo de entrada (micrófono, línea, etc.)
- [ ] Selección de dispositivo de salida (audífonos, bocinas, etc.)
- [ ] Lista de dispositivos de audio disponibles en cada PC
- [ ] Latencia < 50ms para audio en tiempo real
- [ ] Buffer ajustable (calidad vs latencia)

### 1.5 Configuración
- [ ] Inicio automático con el sistema operativo
- [ ] Conexión automática a PC específico
- [ ] Activar/Desactivar cada funcionalidad individualmente:
  - Solo Mouse
  - Solo Teclado
  - Mouse + Teclado
  - Portapapeles
  - Audio (entrada/salida por separado)
- [ ] Configuración de posición de pantallas (drag & drop como Windows)
- [ ] Perfiles de configuración
- [ ] Tema claro/oscuro

## 2. Requisitos Técnicos

### 2.1 Plataformas
- [ ] Windows 10 y 11 (x64)
- [ ] Linux (Omarchy Linux, Ubuntu, Arch, etc.)
- [ ] Versión portable (sin instalador) opcional

### 2.2 Rendimiento
- [ ] Uso de CPU < 5% en reposo
- [ ] Uso de RAM < 200MB
- [ ] Mouse/teclado: < 16ms de latencia
- [ ] Audio: < 50ms de latencia
- [ ] Portapapeles: < 200ms de latencia

### 2.3 Seguridad
- [ ] Conexiones cifradas (SSL/TLS)
- [ ] Autenticación de dispositivos (PIN o fingerprint)
- [ ] Aislamiento de red local (no accesible desde WAN)

### 2.4 Red
- [ ] Protocolo TCP para mouse/teclado/portapapeles (confiabilidad)
- [ ] Protocolo UDP para audio (baja latencia)
- [ ] Compresión de audio (Opus codec)
- [ ] Detección automática de red (mDNS)
- [ ] Reconexión automática

## 3. UI/UX

- [ ] Interfaz intuitiva y minimalista
- [ ] Configuración de pantallas drag & drop
- [ ] Indicador visual de conexión
- [ ] Notificaciones de eventos (conexión, desconexión, errores)
- [ ] Barra de tareas / system tray
- [ ] Atajos de teclado para cambiar entre PCs
