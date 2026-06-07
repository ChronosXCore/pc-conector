# 🖥️ PC Conector - Aplicación (Tauri & React)

Esta carpeta contiene el código fuente completo del frontend (React 19 + TypeScript + Vite) y del backend (Rust + Tauri 2) de la aplicación **PC Conector**.

## 🚀 Inicio Rápido

1. **Requisitos del sistema**:
   Sigue las instrucciones detalladas de instalación de dependencias nativas del sistema en el [README.md principal del repositorio](../README.md).

2. **Instalar dependencias de Node.js**:
   ```bash
   npm install
   ```

3. **Ejecutar en modo de desarrollo**:
   ```bash
   npm run tauri dev
   ```

## 📂 Estructura de código
- `/src`: Archivos del frontend React (pantallas, diseño CSS, componentes, iconos).
- `/src-tauri`: Código Rust para la red QUIC, mDNS local discovery, emulación de mouse/teclado y captura de audio.
