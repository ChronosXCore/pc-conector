<div align="center">

# 📋 PC Conector — Requisitos del Sistema

[![Back to README](https://img.shields.io/badge/←_Volver_al_README-gray?style=flat-square)](../README.md)

</div>

---

## 🗂️ Tabla de Contenidos

- [Funcionalidades Principales](#-funcionalidades-principales)
- [Requisitos Técnicos](#-requisitos-técnicos)
- [UI/UX](#-uiux)

---

## ⭐ Funcionalidades Principales

### 1.1 📡 Descubrimiento Automático en Red

| Requisito | Prioridad | Estado |
|-----------|:---------:|:------:|
| Usar mDNS/Bonjour para detectar otras instancias en la LAN | 🔴 Alta | ✅ |
| Mostrar lista de PCs con nombre, IP y SO | 🔴 Alta | ✅ |
| Conexión automática opcional basada en configuración | 🟡 Media | ✅ |

---

### 1.2 📋 Compartir Portapapeles

| Requisito | Prioridad | Estado |
|-----------|:---------:|:------:|
| Monitorear cambios en el portapapeles local | 🔴 Alta | ✅ |
| Enviar cambios a PCs remotos en tiempo real | 🔴 Alta | ✅ |
| Sincronizar texto plano | 🔴 Alta | ✅ |
| Sincronizar imágenes (configurable) | 🟡 Media | 🔄 |
| Sincronizar archivos (configurable) | 🟢 Baja | 🔄 |
| Latencia < 200ms | 🔴 Alta | ✅ |

---

### 1.3 🖱️ Compartir Mouse y Teclado

| Requisito | Prioridad | Estado |
|-----------|:---------:|:------:|
| Movimiento del mouse entre pantallas (como monitor adicional) | 🔴 Alta | ✅ |
| Click y arrastrar entre PCs | 🔴 Alta | ✅ |
| Teclado compartido (escribir en el PC remoto) | 🔴 Alta | ✅ |
| Configuración de posición de pantallas (drag & drop) | 🔴 Alta | ✅ |
| Soporte para múltiples monitores en cada PC | 🟡 Media | ✅ |
| Latencia de entrada < 16ms (60fps) | 🔴 Alta | ✅ |

---

### 1.4 🔊 Compartir Audio

| Requisito | Prioridad | Estado |
|-----------|:---------:|:------:|
| Transmisión de audio desde micrófono de PC A a PC B | 🔴 Alta | ✅ |
| Transmisión de audio del sistema de PC A a PC B | 🟡 Media | ✅ |
| Bidireccional (ambos sentidos simultáneamente) | 🟡 Media | ✅ |
| Selección de dispositivo de entrada | 🔴 Alta | ✅ |
| Selección de dispositivo de salida | 🔴 Alta | ✅ |
| Latencia < 50ms | 🔴 Alta | ✅ |
| Buffer ajustable (calidad vs latencia) | 🟡 Media | ✅ |

---

### 1.5 ⚙️ Configuración

| Requisito | Prioridad | Estado |
|-----------|:---------:|:------:|
| Inicio automático con el sistema operativo | 🟡 Media | ✅ |
| Conexión automática a PC específico | 🟡 Media | ✅ |
| Activar/Desactivar funciones individualmente | 🔴 Alta | ✅ |
| Configuración de posición de pantallas (drag & drop) | 🔴 Alta | ✅ |
| Perfiles de configuración | 🟡 Media | ✅ |
| Tema claro/oscuro | 🟢 Baja | ✅ |

---

## 🔧 Requisitos Técnicos

### 2.1 Plataformas

| Plataforma | Requerido | Estado |
|-----------|:---------:|:------:|
| Windows 10 (x64) | ✅ | ✅ |
| Windows 11 (x64) | ✅ | ✅ |
| Linux (Arch / CachyOS / Omarchy) | ✅ | ✅ |
| Linux (Ubuntu / Fedora) | 🟡 | ✅ |
| Versión portable sin instalador | 🟢 | 🔄 |

### 2.2 Rendimiento

| Métrica | Objetivo | Estado |
|---------|:--------:|:------:|
| Uso de CPU en reposo | < 5% | ✅ |
| Uso de RAM | < 200MB | ✅ |
| Latencia Mouse/Teclado | < 16ms | ✅ |
| Latencia Audio | < 50ms | ✅ |
| Latencia Portapapeles | < 200ms | ✅ |

### 2.3 Seguridad

| Requisito | Estado |
|-----------|:------:|
| Conexiones cifradas (TLS) | ✅ |
| Autenticación de dispositivos (PIN/fingerprint) | ✅ |
| Aislamiento de red local (no accesible desde WAN) | ✅ |

### 2.4 Red

| Requisito | Estado |
|-----------|:------:|
| TCP para mouse/teclado/portapapeles | ✅ |
| UDP para audio (baja latencia) | ✅ |
| Codec Opus para audio | ✅ |
| Detección automática (mDNS) | ✅ |
| Reconexión automática | ✅ |

---

## 🎨 UI/UX

| Requisito | Estado |
|-----------|:------:|
| Interfaz intuitiva y minimalista | ✅ |
| Configuración de pantallas drag & drop | ✅ |
| Indicador visual de conexión | ✅ |
| Notificaciones de eventos (conexión, desconexión, errores) | ✅ |
| Barra de tareas / system tray | ✅ |
| Atajos de teclado para cambiar entre PCs | 🔄 En progreso |
| Tema oscuro/claro | ✅ |
| Panel de rendimiento en vivo | ✅ |

---

### Leyenda

| Símbolo | Significado |
|:-------:|-------------|
| ✅ | Completado |
| 🔄 | En progreso / Planificado |
| 🔴 | Prioridad Alta |
| 🟡 | Prioridad Media |
| 🟢 | Prioridad Baja |

---

<div align="center">

[← Volver al README](../README.md) · [Arquitectura →](ARCHITECTURE.md) · [Tech Stack →](TECH_STACK.md)

</div>
