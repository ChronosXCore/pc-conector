# 📖 Instrucciones para Compilar PC Conector

## Requisitos del Sistema

1. **Rust** (ya instalado en C:)
2. **Node.js + npm** (ya instalado)
3. **MinGW-w64** (NECESARIO para compilar) o **Visual Studio Build Tools**

## Opción 1: Instalar MinGW-w64 (Recomendada, ~200MB)

1. Descarga MSYS2 desde: https://www.msys2.org/
2. Instálalo (puedes instalarlo en F: para no ocupar C:)
3. Abre MSYS2 y ejecuta:
```bash
pacman -S mingw-w64-x86_64-gcc
```
4. Añade `F:/msys64/mingw64/bin` a tu PATH del sistema

## Opción 2: Visual Studio Build Tools (~500MB)

1. Descarga desde: https://visualstudio.microsoft.com/es/downloads/#build-tools-for-visual-studio
2. Durante la instalación, selecciona "Desarrollo de escritorio con C++"
3. Puedes cambiar la ubicación a F: durante la instalación

## Compilar el Proyecto

Una vez instalado MinGW-w64, abre una terminal y ejecuta:

```bash
# Ir al proyecto
cd F:\pc-conector-project\src-tauri

# Configurar Rust (solo la primera vez)
set RUSTUP_HOME=F:\Programas Desarrollados\Pc conector\rustup-home
set CARGO_HOME=F:\Programas Desarrollados\Pc conector\cargo-cache

# Compilar
cargo build
```

O desde PowerShell:
```powershell
cd F:\pc-conector-project\src-tauri
$env:RUSTUP_HOME="F:\Programas Desarrollados\Pc conector\rustup-home"
$env:CARGO_HOME="F:\Programas Desarrollados\Pc conector\cargo-cache"
cargo build
```

## Para Ejecutar en Modo Desarrollo

```bash
cd F:\pc-conector-project
npm run tauri dev
```

## Estructura del Proyecto

```
pc-conector/
├── docs/                 # Documentación
│   ├── PROJECT_OVERVIEW.md
│   ├── VISION.md
│   ├── ARCHITECTURE.md
│   ├── REQUIREMENTS.md
│   ├── TECH_STACK.md
│   └── PROGRESS.md
├── src/                  # Frontend React
│   ├── App.tsx           # Componente principal
│   ├── App.css           # Estilos
│   ├── main.tsx          # Punto de entrada
│   └── ...
├── src-tauri/            # Backend Rust
│   ├── src/
│   │   ├── main.rs       # Entry point
│   │   ├── lib.rs        # Módulo principal
│   │   ├── config.rs     # Configuración
│   │   ├── discovery.rs  # Descubrimiento mDNS
│   │   ├── network.rs    # Comunicación QUIC
│   │   ├── clipboard.rs  # Portapapeles
│   │   ├── input.rs      # Mouse/Teclado
│   │   └── audio.rs      # Audio
│   ├── Cargo.toml        # Dependencias Rust
│   └── tauri.conf.json   # Config Tauri
├── package.json
└── INSTRUCCIONES.md
```
