# 📊 PC Conector - Progreso del Desarrollo

## Estado Actual: 🟡 PLANIFICACIÓN

### Fase 0: Planificación y Diseño
- [x] Investigación de proyectos open-source existentes
- [x] Definición de visión y stack tecnológico
- [x] Creación de documentación del proyecto
- [ ] Preguntas al usuario para definir alcance
- [ ] Diseño de arquitectura detallada
- [ ] Definición de API interna

### Fase 1: Fundación
- [ ] Configuración del proyecto Tauri con dependencias
- [ ] Implementación del módulo de red (mDNS + WebSocket)
- [ ] Prueba de comunicación básica entre 2 instancias
- [ ] UI base (conexión, panel de estado)

### Fase 2: Portapapeles
- [ ] Implementación de monitoreo de clipboard (arboard)
- [ ] Sincronización de clipboard via WebSocket
- [ ] Interfaz de configuración de clipboard
- [ ] Pruebas de sincronización en tiempo real

### Fase 3: Mouse y Teclado
- [ ] Implementación de captura de eventos globales (rdev)
- [ ] Implementación de simulación de entrada (enigo)
- [ ] Sistema de coordenadas virtuales y grilla de monitores
- [ ] Interfaz drag & drop para posición de monitores
- [ ] Transferencia de cursor entre PCs
- [ ] Pruebas de input remoto

### Fase 4: Audio
- [ ] Captura de audio local (CPAL)
- [ ] Codec Opus para compresión
- [ ] Streaming UDP con control de jitter
- [ ] Reproducción de audio remoto
- [ ] Interfaz de selección de dispositivos
- [ ] Pruebas de latencia y calidad

### Fase 5: Configuración y Pulido
- [ ] Auto-inicio con el sistema operativo
- [ ] Gestión de perfiles de conexión
- [ ] Cifrado de comunicación
- [ ] Pruebas en Windows y Linux
- [ ] Optimización de rendimiento
- [ ] Instaladores y distribución

---

## Registro de Cambios

| Fecha | Cambio | Estado |
|------|--------|--------|
| - | Creación del proyecto | ✅ |
| - | Documentación inicial | ✅ |
