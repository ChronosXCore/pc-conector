# 🤖 PROMPT PARA EL AGENTE DE IA (Linux / Arch Linux)

Copia todo el texto a continuación y pégalo en el chat de tu agente de IA en la otra computadora (la que tiene Linux). Este prompt está diseñado para guiarlo paso a paso en la descarga, instalación de dependencias y ejecución de la aplicación.

---

### INICIO DEL PROMPT PARA EL AGENTE:

"Hola. Eres un agente de desarrollo de software en una PC Linux (basada en Arch Linux/CachyOS). Tu objetivo es poner en marcha la aplicación de escritorio **PC Conector (NetBridge)** (Tauri 2 + React 19 + TypeScript + Rust) en esta máquina.

Puedes obtener el código de dos maneras:
1. **Opción A (Recomendada - GitHub)**: Clonando el repositorio desde GitHub si el usuario te proporciona el enlace.
2. **Opción B (Local - ZIP)**: Si hay un archivo `pc-conector.zip` en tu directorio de trabajo, lo descomprimirás.

Sigue estos pasos detallados para completar la tarea de forma autónoma:

---

## 🛠️ PASO 1 — Instalar prerrequisitos y dependencias de Tauri (Arch Linux)

Tauri requiere `webkit2gtk-4.1` y otras librerías nativas para compilar el backend de Rust y renderizar el frontend. Ejecuta el siguiente comando para instalarlas todas en Arch Linux / CachyOS / Manjaro:

```bash
sudo pacman -S --needed base-devel nodejs npm webkit2gtk-4.1 libappindicator-gtk3 librsvg openssl git
```

*(Si el sistema es Ubuntu/Debian en lugar de Arch, usa: `sudo apt update && sudo apt install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf pkg-config build-essential libssl-dev libgtk-3-dev nodejs npm git`)*

---

## 🦀 PASO 2 — Instalar/Verificar Rust

Verifica si Rust está instalado ejecutando `rustc --version` y `cargo --version`. Si no lo está, instálalo con:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

---

## 📂 PASO 3 — Obtener y preparar el proyecto

Pregúntale al usuario si prefiere darte la URL de su repositorio de GitHub para clonarlo, o si prefiere que uses el archivo `pc-conector.zip` si existe en tu directorio actual.

- **Si clonas desde GitHub**:
  ```bash
  git clone <URL_DEL_REPOSITORIO> pc-conector-repo
  cd pc-conector-repo/pc-conector
  ```
- **Si usas el ZIP local**:
  ```bash
  unzip pc-conector.zip -d pc-conector-temp
  cd pc-conector-temp/pc-conector
  ```

Asegúrate de que estás en la carpeta del proyecto donde se encuentra el archivo `package.json` y la carpeta `src-tauri/`.

---

## 📦 PASO 4 — Instalar dependencias de Node.js

Instala las dependencias del frontend:

```bash
npm install
```

---

## 🚀 PASO 5 — Ejecutar en modo desarrollo

Inicia la aplicación en modo desarrollo:

```bash
npm run tauri dev
```

Este comando:
1. Iniciará el servidor de desarrollo de Vite (React 19).
2. Compilará el backend en Rust mediante Cargo (la primera compilación puede tardar de 5 a 15 minutos mientras descarga y compila los crates de Rust).
3. Lanzará la ventana nativa de la aplicación.

---

## 🔍 PASO 6 — Verificación de funcionamiento
Una vez que abra la ventana de la aplicación:
- Confirma que se muestre el nuevo diseño con el alternador de tema claro/oscuro (☀️/🌙) en la parte superior derecha.
- Abre la sección **Red & Rendimiento** y verifica que muestre la IP local del equipo y empiece a graficar la latencia del ping en tiempo real.
- Confirma que no haya errores de consola en la terminal.

¡Manos a la obra! Infórmame de tu progreso en cada paso."
