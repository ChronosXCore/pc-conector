# 🖥️ PC Conector - Visión del Proyecto

## Visión General

**PC Conector** es una aplicación de escritorio multiplataforma que permite conectar dos o más PCs a través de la red local (WiFi/LAN) para compartir:

- **Portapapeles**: Copia en un PC, pega en el otro sin fricción
- **Mouse y Teclado**: Controla múltiples PCs como si fueran monitores adicionales
- **Audio**: Stream de audio bidireccional entre equipos

Todo con una interfaz intuitiva, configurable y con detección automática.

---

## Problema a Resolver

Los usuarios con múltiples PCs necesitan:
1. **Cambiar entre equipos físicamente** (mover mouse, teclado)
2. **Compartir archivos/texto** entre equipos manualmente (USB, cloud)
3. **Configurar audio** en múltiples equipos de forma separada

## Solución

PC Conector unifica la experiencia de múltiples PCs en un solo flujo de trabajo:

- ✅ Detección automática de PCs en la red
- ✅ Compartir portapapeles en tiempo real
- ✅ Control unificado de mouse y teclado
- ✅ Streaming de audio bidireccional
- ✅ Configuración visual de posición de pantallas (drag & drop)
- ✅ Soporte Windows + Linux (Omarchy Linux)

## Stack Tecnológico

| Capa | Tecnología |
|------|-----------|
| Frontend | React + TypeScript + Vite |
| Backend | Rust (Tauri) |
| GUI Framework | Tauri v2 |
| Red | TCP/UDP personalizado + mDNS/Zeroconf |
| Audio | CPAL + PortAudio |
| Input Simulation | enigo / rdev (Rust) |
| Clipboard | arboard (Rust) |

---

## Principios de Diseño

1. **Rendimiento**: Mínima latencia, máxima capacidad de respuesta
2. **Intuitividad**: Configuración visual, sin archivos de configuración complejos
3. **Configurabilidad**: Cada función es opcional y configurable
4. **Multiplataforma**: Windows y Linux con la misma experiencia
5. **Seguridad**: Comunicación cifrada y autenticación de dispositivos
