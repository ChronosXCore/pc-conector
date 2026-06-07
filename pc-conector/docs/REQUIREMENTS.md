# PC Conector - Requirements

## 📋 Funcionalidades Core

### 1. Descubrimiento de Red
- [ ] Buscar automáticamente PCs en la red WiFi local
- [ ] Usar mDNS para descubrimiento de peers
- [ ] Mostrar PCs disponibles con nombre e IP
- [ ] Conexión manual por IP como fallback

### 2. Sincronización de Portapapeles
- [ ] Detectar cambios en el portapapeles local
- [ ] Enviar contenido al peer conectado
- [ ] Sincronización bidireccional en tiempo real
- [ ] Soporte para texto, imágenes y archivos

### 3. Compartir Mouse y Teclado (KVM over IP)
- [ ] Capturar eventos globales de mouse
- [ ] Capturar eventos globales de teclado
- [ ] Enviar eventos al PC remoto
- [ ] Simular eventos en el PC remoto como si fueran locales
- [ ] Movimiento fluido entre pantallas (edge transitions)
- [ ] Configuración de posición de pantallas (drag & drop)

### 4. Audio Bidireccional
- [ ] Capturar audio del micrófono local y enviarlo al peer
- [ ] Recibir audio del peer y reproducirlo en altavoces locales
- [ ] Selección de dispositivos de audio (micrófono, altavoces)
- [ ] Baja latencia para comunicación en tiempo real
- [ ] Compresión de audio para optimizar ancho de banda

### 5. Configuración
- [ ] Configurar conexión automática al iniciar
- [ ] Elegir qué servicios activar (mouse, teclado, clipboard, audio)
- [ ] Interfaz drag-and-drop para posición de pantallas
- [ ] Configuración de dispositivos de audio
- [ ] Perfiles de configuración múltiples
- [ ] Auto-inicio con el sistema operativo

### 6. Plataformas Soportadas
- [ ] Windows 10/11
- [ ] Linux (Omarchy Linux, Ubuntu, Arch, etc.)
- [ ] Soporte para Wayland y X11 en Linux

## 🎨 Interfaz de Usuario

- [ ] Pantalla principal con estado de conexión
- [ ] Panel de descubrimiento de PCs
- [ ] Configuración de posición de pantallas (drag & drop)
- [ ] Panel de selección de servicios activos
- [ ] Configuración de dispositivos de audio
- [ ] Configuración de auto-inicio
- [ ] Indicadores visuales de latencia/calidad de conexión

## ⚡ Requisitos Técnicos

- [ ] Latencia < 10ms para input (mouse/teclado)
- [ ] Latencia < 50ms para audio
- [ ] Consumo de CPU optimizado
- [ ] Cifrado de datos entre peers
- [ ] Reconexión automática
- [ ] Manejo de errores graceful
