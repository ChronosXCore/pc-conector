# AGENTE — INSTRUCCIONES DE SETUP: PC Conector (Linux)

Eres un agente de IA en una PC Linux. Tienes en esta misma carpeta un archivo `pc-conector.zip` con el código fuente completo de una aplicación **Tauri 2 + React 19 + TypeScript**. 

Tu tarea es **descomprimir, instalar dependencias y ejecutar la app** en modo desarrollo. Sigue exactamente estos pasos en orden.

---

## PASO 1 — Verificar prerrequisitos del sistema

Ejecuta estos comandos para ver qué falta:

```bash
node --version
npm --version
rustc --version
cargo --version
```

**Si node/npm no están instalados**, instálalos:
```bash
# Ubuntu/Debian/Mint:
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Arch/Manjaro:
sudo pacman -S nodejs npm

# Fedora:
sudo dnf install -y nodejs npm
```

**Si Rust no está instalado**, instálalo:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

---

## PASO 2 — Instalar dependencias del sistema para Tauri

Tauri requiere webkit2gtk y otras librerías nativas. Instálalas:

```bash
# Ubuntu/Debian/Mint:
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  pkg-config \
  build-essential \
  libssl-dev \
  libgtk-3-dev

# Arch/Manjaro:
sudo pacman -S --needed \
  webkit2gtk-4.1 \
  libappindicator-gtk3 \
  librsvg \
  base-devel \
  openssl

# Fedora:
sudo dnf install -y \
  webkit2gtk4.1-devel \
  openssl-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel
```

---

## PASO 3 — Descomprimir el proyecto

La carpeta donde está este archivo es tu directorio de trabajo. Descomprime aquí:

```bash
# Situate en la carpeta donde está este archivo y el ZIP:
cd /ruta/donde/esta/este/archivo

unzip pc-conector.zip -d pc-conector
cd pc-conector
```

La estructura extraída debe verse así:
```
pc-conector/
├── src/              ← Frontend React + TypeScript
├── src-tauri/        ← Backend Rust + Tauri config
├── public/
├── index.html
├── package.json
├── vite.config.ts
└── tsconfig.json
```

---

## PASO 4 — Instalar dependencias de Node

```bash
# Dentro de la carpeta pc-conector/:
npm install
```

---

## PASO 5 — Ejecutar la app en modo desarrollo

```bash
npm run tauri dev
```

Esto hará automáticamente:
1. Levanta el servidor Vite (frontend React)
2. Compila el backend Rust con Cargo
3. Abre la ventana de la app

> ⏱️ La **primera compilación de Rust** tarda entre 5-15 minutos. Las siguientes son mucho más rápidas (incremental).

---

## SOLUCIÓN DE PROBLEMAS COMUNES

### Error: `webkit2gtk-4.1` not found
```bash
# Prueba con la versión 4.0:
sudo apt install -y libwebkit2gtk-4.0-dev
# Y edita src-tauri/Cargo.toml si es necesario
```

### Error: `tauri-cli` no encontrado
```bash
npm install  # asegúrate de haber corrido esto primero
```

### Error de permisos en cargo
```bash
source "$HOME/.cargo/env"
# O cierra y vuelve a abrir la terminal
```

### La app abre pero muestra pantalla en blanco
```bash
# El frontend no se compiló. Asegúrate de que Vite corre:
npm run dev   # En una terminal aparte para verificar
```

---

## NOTAS TÉCNICAS DEL PROYECTO

- **Framework**: Tauri 2 + React 19 + TypeScript
- **Build tool**: Vite 8
- **Puerto dev frontend**: `http://localhost:5173`
- **Backend**: Rust (se compila automáticamente)
- **Funcionalidad**: Conecta dos PCs en red local para compartir ratón, teclado, portapapeles y audio

---

Una vez que la app corra, ¡el diseño ya incluye modo claro/oscuro, iconos SVG premium y animaciones fluidas! 🚀
