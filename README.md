# 🖥️ PC Conector 🔗

**PC Conector** (NetBridge) es una aplicación de escritorio moderna, ultraliviana y multiplataforma diseñada para unificar tu flujo de trabajo al conectar dos computadoras en tu red local. Permite compartir de forma fluida el ratón, teclado, portapapeles y audio bidireccional en tiempo real.

Desarrollada con **Tauri 2 (Rust) + React 19 + TypeScript**, cuenta con un diseño premium y de alto rendimiento.

---

## ✨ Características Principales

- 🌙 **Modo Oscuro / Claro**: Alternador rápido de tema (sol y luna) persistente en la barra superior con transiciones fluidas de CSS.
- 📡 **Panel de Red y Rendimiento**:
  - Visualización del **Hostname** e **IPs locales** de tu máquina con un botón de copiado rápido (📋 Copiar / ✓ Copiada).
  - **Gráfica de barras de latencia en vivo** con actualización automática cada 3 segundos.
  - Indicador dinámico de estado por colores (🟢 Excelente, 🟢 Bueno, 🟡 Regular, 🟠 Lento, 🔴 Sin respuesta).
  - Métricas de red detalladas: Latencia Actual, Promedio, Mínimo y Máximo.
  - Configuración del destino del ping (por defecto `8.8.8.8`).
- 🎯 **Detección Inteligente de Dispositivos**:
  - Clasificación automática y nombres descriptivos con íconos premium para cada tipo de equipo (Celulares 📱, Laptops 💻, PCs 🖥️, Impresoras 🖨️, TVs 📺, Routers 📡, etc.).
  - Detección precisa de marcas conocidas mediante la MAC o el Hostname (Samsung, Xiaomi/Poco, ZTE, Motorola, Realme, OnePlus, etc.).
- 🖱️ **Compartir Teclado y Mouse**: Control sin interrupciones deslizando el cursor más allá del borde de tu pantalla.
- 📋 **Sincronización del Portapapeles**: Copia texto en un ordenador y pégalo instantáneamente en el otro.
- 🔊 **Transmisión de Audio**: Comparte el sonido de tu micrófono o salida de audio entre dispositivos con bajísima latencia.

---

## 📂 Estructura del Repositorio

```text
.
├── pc-conector/        # Código fuente completo (Frontend React + Backend Rust/Tauri)
├── para-linux/         # Instrucciones y scripts de instalación rápida en Linux (Arch/Ubuntu/Fedora)
│   └── AGENTE_PROMPT.md # Prompt estructurado para que un agente de IA haga la instalación por ti
├── docs/               # Documentación detallada del proyecto (Arquitectura, Progreso, etc.)
└── Logo.png            # Logotipo de la aplicación
```

---

## 🚀 Requisitos e Instalación

### 🪟 En Windows

Para ejecutar la aplicación en modo desarrollo en Windows:

1. **Prerrequisitos**:
   - Tener instalado [Node.js](https://nodejs.org/) (incluye `npm`).
   - Tener instalado Rust mediante [Rustup](https://rustup.rs/).
   - MinGW-w64 (GCC) o Visual Studio Build Tools.
2. **Ejecutar**:
   Abre una terminal PowerShell en la carpeta raíz y corre:
   ```powershell
   cd pc-conector
   npm install
   npm run tauri dev
   ```

---

### 🐧 En Linux (Arch Linux / CachyOS / Ubuntu / Fedora)

Hemos preparado una guía completa y automatizada para Linux. Si estás utilizando un agente de IA en la máquina de destino (Linux), puedes pasarle el archivo `/para-linux/AGENTE_PROMPT.md` para que configure todo de forma autónoma.

#### Instalación Manual en Arch Linux / CachyOS (Omarchy Linux):
1. **Instalar dependencias del sistema**:
   ```bash
   sudo pacman -S --needed base-devel nodejs npm webkit2gtk-4.1 libappindicator-gtk3 librsvg openssl
   ```
2. **Instalar Rust** (si no lo tienes):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```
3. **Instalar paquetes y ejecutar**:
   ```bash
   cd pc-conector
   npm install
   npm run tauri dev
   ```

---

## 🤖 Uso con Agentes de IA (Instalación 1-Clic)

Si tienes un agente de codificación de IA en tu otra máquina (por ejemplo, en tu laptop con Linux), puedes copiar el contenido de [para-linux/AGENTE_PROMPT.md](file:///F:/Programas%20Desarrollados/Pc%20conector/para-linux/AGENTE_PROMPT.md) y pegarlo en su chat. El agente se encargará de clonar el código de tu repositorio de GitHub, instalar los paquetes nativos necesarios, compilar el backend en Rust y dejar la aplicación funcionando.

---

## 🎨 Diseño Visual & Premium

El diseño se ha perfeccionado usando **CSS puro de alto nivel** con animaciones fluidas, variables adaptables al tema claro/oscuro (Glassmorphism, sombras neon y bordes suaves), garantizando una experiencia visual premium y rápida en cualquier resolución de pantalla.
