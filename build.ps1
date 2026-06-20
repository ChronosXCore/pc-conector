# ============================================================
# build.ps1  -  NetBridge - Script de Compilacion Completo
# Uso: .\build.ps1          -> compila el instalador
#      .\build.ps1 -Dev     -> inicia en modo desarrollo
# ============================================================
param([switch]$Dev)

# Configurar rutas de Rust/Cargo con msys64 y wrapper cmake para compat con CMake 4.x
$env:RUSTUP_HOME = "F:\Programas Desarrollados\Pc conector\rustup-home"
$env:CARGO_HOME  = "F:\Programas Desarrollados\Pc conector\cargo-cache"
$env:CC          = "F:\msys64\mingw64\bin\gcc.exe"
$env:CXX         = "F:\msys64\mingw64\bin\g++.exe"
$env:AR          = "F:\msys64\mingw64\bin\ar.exe"
$env:CC_x86_64_pc_windows_gnu = "F:\msys64\mingw64\bin\gcc.exe"

# cmake-bin tiene un wrapper que inyecta -DCMAKE_POLICY_VERSION_MINIMUM=3.5
# necesario para compilar audiopus_sys (Opus) con CMake 4.x
$env:PATH = "F:\Programas Desarrollados\Pc conector\cmake-bin;" +
            "F:\msys64\mingw64\bin;" +
            "F:\Programas Desarrollados\Pc conector\cargo-cache\bin;" +
            "F:\Programas Desarrollados\Pc conector\rustup-home\toolchains\stable-x86_64-pc-windows-gnu\bin;" +
            $env:PATH

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "   NetBridge - Build Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Cargo: $(cargo --version)" -ForegroundColor Green
Write-Host ""

Set-Location "F:\Programas Desarrollados\Pc conector\pc-conector"

if ($Dev) {
    Write-Host "Iniciando en modo DESARROLLO..." -ForegroundColor Yellow
    npx tauri dev
} else {
    Write-Host "Compilando instalador (esto puede tardar 5-15 minutos)..." -ForegroundColor Yellow
    npx tauri build
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "BUILD EXITOSO!" -ForegroundColor Green
        $installer = Get-ChildItem "src-tauri\target\release\bundle" -Recurse -Filter "*.msi" | Select-Object -First 1
        if ($installer) {
            Write-Host "Instalador MSI: $($installer.FullName)" -ForegroundColor Cyan
        }
        $exe = Get-ChildItem "src-tauri\target\release\bundle" -Recurse -Filter "*.exe" | Where-Object { $_.Name -like "*setup*" -or $_.Name -like "*install*" } | Select-Object -First 1
        if ($exe) {
            Write-Host "Instalador EXE: $($exe.FullName)" -ForegroundColor Cyan
        }
    } else {
        Write-Host "BUILD FALLIDO. Revisa los errores de arriba." -ForegroundColor Red
    }
}
