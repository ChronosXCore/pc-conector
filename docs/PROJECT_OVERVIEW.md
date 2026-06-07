<div align="center">

# 📦 PC Conector — Visión General del Proyecto

[![Back to README](https://img.shields.io/badge/←_Volver_al_README-gray?style=flat-square)](../README.md)
[![Visión](https://img.shields.io/badge/Ver_Visión_Completa-7c3aed?style=flat-square)](VISION.md)

</div>

---

## 🎯 Objetivo del Proyecto

Crear una aplicación de escritorio **gratuita, open-source y multiplataforma** que permita conectar dos PCs a través de la red local (WiFi/Ethernet) para compartir recursos de forma transparente y con mínima latencia.

---

## 🌟 ¿Qué puede hacer PC Conector?

| Función | Descripción | Latencia |
|---------|-------------|:--------:|
| 📋 **Portapapeles** | Copia en un PC, pega en el otro sin intervención | < 200ms |
| 🖱️ **Mouse compartido** | Controla ambos PCs como si fueran uno solo | < 16ms |
| ⌨️ **Teclado compartido** | Escribe en cualquier PC desde un solo teclado | < 16ms |
| 🔄 **Movimiento entre pantallas** | Arrastra el mouse de un monitor al otro | < 16ms |
| 🔊 **Audio bidireccional** | Micrófono y altavoces entre PCs | < 50ms |
| 📡 **Auto-descubrimiento** | Detecta automáticamente otros PCs en la red | Instantáneo |

---

## 💡 Inspiración

| Proyecto | Lo que inspira |
|----------|---------------|
| **Input Leap / Barrier / Synergy** | Compartir mouse y teclado entre PCs |
| **KDE Connect** | Sincronización de portapapeles |
| **SonoBus** | Streaming de audio de baja latencia |
| **Microsoft Mouse Without Borders** | Experiencia fluida de múltiples PCs |

---

## 👥 Público Objetivo

| Perfil | Caso de Uso |
|--------|-------------|
| 💻 Desarrolladores | Windows + Linux simultáneamente |
| 🎮 Gamers | Setup multi-PC con un solo input |
| 🏠 Usuarios domésticos | PC de escritorio + laptop |
| 🐧 Entusiastas Linux | Omarchy Linux / Arch |
| 🏢 Profesionales | Home office / trabajo híbrido |

---

## 🛠️ Stack Tecnológico

| Componente | Tecnología | Por qué |
|------------|-----------|---------|
| **Lenguaje Principal** | Rust 🦀 | Rendimiento, seguridad, concurrencia nativa |
| **Framework UI** | Tauri 2.x | Liviano (WebView nativo), seguro, perfecto para Rust |
| **Frontend** | React + TypeScript | UI moderna, flexible, ecosistema maduro |
| **Mouse/Teclado** | TCP personalizado + rdev/enigo | Basado en Input Leap |
| **Portapapeles** | `arboard` | Multiplataforma, mantenido por 1Password |
| **Audio** | `cpal` + `ringbuf` + UDP | Baja latencia, control fino de dispositivos |
| **Descubrimiento** | mDNS/Bonjour (`mdns-sd`) | Detección automática sin config manual |
| **Config** | JSON + UI interactiva | Sencillo y portable |

---

## 🎯 Resultado Logrado

✅ Se instala fácilmente en Windows y Linux (Omarchy)  
✅ Al abrirse, detecta automáticamente otras instancias en la red  
✅ Permite configurar la posición de las pantallas arrastrando (como en Windows)  
✅ Sincroniza el portapapeles en tiempo real  
✅ Permite mover el mouse entre PCs suavemente  
✅ Transmite audio bidireccional con latencia < 50ms  
✅ Se inicia con el sistema operativo (opcional)  
✅ Es completamente configurable (qué funciones activar, a qué PC conectarse)  
✅ Diseño premium con modo oscuro/claro, glassmorphism y animaciones CSS

---

<div align="center">

[← Volver al README](../README.md) · [Visión Completa →](VISION.md) · [Arquitectura →](ARCHITECTURE.md)

</div>
