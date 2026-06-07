# PC Conector 🖥️🔗🖥️

## Visión del Proyecto

Crear una aplicación de escritorio **gratuita, open-source y multiplataforma** que permita conectar dos PCs a través de la red local (WiFi/Ethernet) para compartir:

- ✅ **Portapapeles** (copiar en un PC, pegar en otro)
- ✅ **Mouse y teclado** (controlar ambos PCs como si fueran un solo sistema)
- ✅ **Movimiento entre pantallas** (arrastrar el mouse de un monitor a otro)
- ✅ **Audio bidireccional** (micrófono y altavoces entre PCs)
- ✅ **Configuración total** de todas las funcionalidades

## Inspiración

El proyecto se inspira en:
- **Input Leap / Barrier / Synergy** → para compartir mouse y teclado
- **KDE Connect** → para sincronización de portapapeles
- **SonoBus** → para streaming de audio de baja latencia
- **Microsoft Mouse Without Borders** → para la experiencia de múltiples PCs

## Público Objetivo

- Usuarios con múltiples PCs en casa u oficina
- Desarrolladores que trabajan con varios equipos
- Entusiastas de Linux (especialmente Omarchy Linux) y Windows
- Cualquier persona que quiera optimizar su flujo de trabajo entre computadoras

## Stack Tecnológico (Propuesto)

| Componente | Tecnología | Razón |
|------------|-----------|-------|
| **Lenguaje Principal** | Rust 🦀 | Rendimiento, seguridad, concurrencia nativa, compilado |
| **Framework UI** | Tauri 2.x | Liviano (usa WebView nativo), seguro, perfecto para Rust |
| **Frontend** | React + TypeScript | UI moderna, flexible, ecosistema maduro |
| **Mouse/Teclado** | Protocolo personalizado sobre TCP + IPC nativo | Basado en la arquitectura de Input Leap |
| **Portapapeles** | `arboard` + polling loop | Multiplataforma, mantenido por 1Password |
| **Audio** | `cpal` + `ringbuf` + UDP | Baja latencia, control fino de dispositivos |
| **Descubrimiento** | mDNS/Bonjour (`mdns-sd`) | Detección automática en red local |
| **Configuración** | JSON/YAML + UI interactiva | Sencillo y portable |

## Resultado Esperado

Una aplicación que:
1. Se instala fácilmente en Windows y Linux (Omarchy)
2. Al abrirse, detecta automáticamente otras instancias en la red
3. Permite configurar la posición de las pantallas arrastrando (como en Windows)
4. Sincroniza el portapapeles en tiempo real
5. Permite mover el mouse entre PCs suavemente
6. Transmite audio bidireccional con latencia < 50ms
7. Se inicia con el sistema operativo (opcional)
8. Es completamente configurable (qué funciones activar, a qué PC conectarse)
