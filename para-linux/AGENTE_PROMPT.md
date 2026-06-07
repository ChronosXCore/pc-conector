# 🤖 PROMPT PARA EL AGENTE DE IA (Linux / Ubuntu / Arch)

Copia todo el texto a continuación y pégalo en el chat de tu agente de IA en la otra computadora (la que tiene Linux). Este prompt está diseñado para guiarlo paso a paso en la descarga, instalación de dependencias, configuración de puertos de red y ejecución autónoma de la aplicación **NetBridge**.

---

### INICIO DEL PROMPT PARA EL AGENTE:

"Hola. Eres un agente de desarrollo de software en una PC Linux (basada en Ubuntu, Debian, Mint o Arch Linux). Tu objetivo es poner en marcha la aplicación de escritorio **NetBridge** (anteriormente *PC Conector*) en esta máquina de forma 100% autónoma.

Esta aplicación está construida con **Tauri 2 (Rust) + React 19 (TypeScript) + CSS Puro**.

Sigue estos pasos detallados para completar la tarea de forma autónoma:

---

## 🛠️ PASO 1 — Instalar dependencias nativas de Tauri y GTK

Tauri necesita librerías de desarrollo nativas para compilar el backend de Rust y renderizar el frontend webview. Ejecuta el comando correspondiente según la distribución instalada:

### En Ubuntu / Debian / Linux Mint:
```bash
sudo apt update && sudo apt install -y \
  build-essential curl libssl-dev libwebkit2gtk-4.1-dev \
  libgtk-3-dev libappindicator3-dev librsvg2-dev patchelf nodejs npm git
```

### En Arch Linux / CachyOS / Manjaro:
```bash
sudo pacman -S --needed base-devel nodejs npm webkit2gtk-4.1 libappindicator-gtk3 librsvg openssl git
```

---

## 🦀 PASO 2 — Instalar/Verificar Rust

Verifica si Rust está instalado ejecutando `rustc --version` y `cargo --version`. Si no lo está o está desactualizado, instálalo ejecutando:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

---

## 📂 PASO 3 — Obtener y actualizar el código fuente

Asegúrate de clonar el repositorio en la última versión de la rama principal (`main`):

```bash
# 1. Clonar el repositorio
git clone https://github.com/ChronosXCore/pc-conector.git
cd pc-conector/pc-conector

# 2. Asegurar que estás en la última versión de la rama principal
git checkout main
git pull origin main
```

Asegúrate de estar posicionado en la subcarpeta `pc-conector` (donde se encuentra `package.json` y la subcarpeta `src-tauri`).

---

## 🛡️ PASO 4 — Configurar puertos de red en el cortafuegos (UFW)

Para permitir el descubrimiento por mDNS y la conexión cifrada directa (QUIC) entre ambas máquinas, abre los puertos necesarios ejecutando:

```bash
sudo ufw allow 9876/udp
sudo ufw allow 5353/udp
sudo ufw reload
sudo ufw status
```

---

## 📦 PASO 5 — Instalar dependencias de Node.js

Instala los módulos de Node necesarios para el frontend web:

```bash
npm install
```

---

## 🚀 PASO 6 — Compilar y ejecutar la aplicación en desarrollo

Inicia el servidor de desarrollo de Vite y el shell de Tauri:

```bash
npm run tauri dev
```

Este comando:
1. Iniciará el servidor de desarrollo local de Vite (React 19).
2. Compilará el backend en Rust mediante Cargo (la primera compilación puede tardar entre 5 y 15 minutos).
3. Lanzará la ventana de la aplicación de escritorio **NetBridge**.

---

## 🔍 PASO 7 — Verificación de funcionamiento

Una vez que la interfaz abra:
1. Confirma que en la barra superior del Dashboard empiece la búsqueda automática local y veas la IP de este dispositivo.
2. Abre la sección de **Pantallas** y confirma que se lean tus pantallas locales.
3. Si el otro PC (Windows) ya tiene abierto NetBridge y están en la misma red local, confirma si se auto-conectan o si puedes conectarlos ingresando la IP manualmente.

Infórmame de tu progreso en cada paso y reporta cuando la ventana esté activa."
